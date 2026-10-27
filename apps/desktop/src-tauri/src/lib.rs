// 前端不使用 Tauri IPC（invoke），而是 HTTP 调用 axum sidecar（rust/server）。
// 本 crate 只负责：随应用启动 sidecar、转发其日志、退出时回收子进程。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::{Manager, RunEvent};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

/// 持有 axum sidecar 子进程，应用退出时 kill，避免残留进程占用端口
struct ServerChild(Mutex<Option<CommandChild>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 对应 tauri.conf.json bundle.externalBin 里的 "binaries/server"
            let (mut rx, child) = app.shell().sidecar("server")?.spawn()?;
            app.manage(ServerChild(Mutex::new(Some(child))));

            // 转发 sidecar 的 stdout/stderr 到终端，方便排查
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(line) => {
                            eprintln!("[server] {}", String::from_utf8_lossy(&line))
                        }
                        CommandEvent::Stderr(line) => {
                            eprintln!("[server] {}", String::from_utf8_lossy(&line))
                        }
                        CommandEvent::Terminated(status) => {
                            eprintln!("[server] terminated: {status:?}")
                        }
                        CommandEvent::Error(err) => eprintln!("[server] error: {err}"),
                        _ => {}
                    }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!());
    // 入口就是最上层：构建失败无处上报，报告 + 非零退出（不走 panic 通道）
    let app = match app {
        Ok(app) => app,
        Err(err) => {
            eprintln!("Tauri 应用构建失败：{err}");
            std::process::exit(1);
        }
    };

    app.run(|app, event| {
        if let RunEvent::Exit = event
            // 锁中毒（持有线程曾 panic）时放弃回收，不再引入 panic
            && let Some(child) =
                app.state::<ServerChild>().0.lock().ok().and_then(|mut guard| guard.take())
        {
            let _ = child.kill();
        }
    });
}
