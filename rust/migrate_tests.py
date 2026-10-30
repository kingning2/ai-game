"""一次性迁移脚本：内联 #[cfg(test)] mod tests {} → 独立 tests 文件 + #[path] 挂载。
用法: python migrate_tests.py [--apply]  (默认 dry-run)
"""
import os, re, io, sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

APPLY = '--apply' in sys.argv

RAW_STR = re.compile(r'(?:b?r)(#*)"')

def find_block_end(src, start):
    i, depth, n = start, 0, len(src)
    while i < n:
        c = src[i]
        # raw string: r#"..."# / br##"..."##（json!/测试数据大量使用）
        if c in 'rb' and (m := RAW_STR.match(src, i)):
            hashes = m.group(1)
            end = src.find('"' + hashes, m.end())
            i = (end + 1 + len(hashes)) if end >= 0 else n
            continue
        if c == '"':
            j = i + 1
            while j < n:
                if src[j] == '\\':
                    j += 2
                    continue
                if src[j] == '"':
                    break
                j += 1
            i = j + 1
            continue
        if c == "'":  # char 字面量或 lifetime（'static 不是字符）
            if src[i:i+2] == "'\\":
                j = i + 3
            elif i + 2 < n and src[i+2] == "'":
                j = i + 2
            else:
                i += 1  # lifetime：跳过引号继续
                continue
            while j < n and src[j] != "'":
                j += 1
            i = j + 1
            continue
        if src[i:i+2] == '//':
            j = src.find('\n', i)
            i = n if j < 0 else j
            continue
        if src[i:i+2] == '/*':
            j = src.find('*/', i + 2)
            i = n if j < 0 else j + 2
            continue
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1

def migrate(path):
    src = io.open(path, encoding='utf-8').read()
    m = re.search(r'#\[cfg\(test\)\]\s*\nmod tests \{', src)
    if not m:
        return f'!! 模式不匹配: {path}'
    brace = src.index('{', m.start())
    end = find_block_end(src, brace)
    body = src[brace + 1:end]
    mod_file = os.path.basename(path)
    if mod_file == 'mod.rs':
        target, path_attr = 'tests.rs', 'tests.rs'
    else:
        target = mod_file[:-3] + '_tests.rs'
        path_attr = target
    tests_src = '//! ' + os.path.basename(path) + ' 的单元测试（由内联 mod tests 迁入）。\n' + body.rstrip() + '\n'
    # 原文件：内联块 → #[path] 挂载
    replacement = '#[cfg(test)]\n#[path = "%s"]\nmod tests;\n' % path_attr
    new_src = src[:m.start()] + replacement + src[end + 1:].lstrip('\n')
    if APPLY:
        io.open(os.path.join(os.path.dirname(path), target), 'w', encoding='utf-8', newline='').write(tests_src)
        io.open(path, 'w', encoding='utf-8', newline='').write(new_src)
    return f'{"✓" if APPLY else "·"} {path} → {target}'

files = []
for root, dirs, names in os.walk('.'):
    dirs[:] = [d for d in dirs if d != 'target']
    for name in names:
        if name.endswith('.rs') and name not in ('tests.rs',) and not name.endswith('_tests.rs'):
            p = os.path.join(root, name)
            src = io.open(p, encoding='utf-8').read()
            if '#[cfg(test)]' in src and 'mod tests {' in src:
                files.append(p)

print(f'{len(files)} 个文件待迁移\n')
for p in sorted(files):
    print(migrate(p))
