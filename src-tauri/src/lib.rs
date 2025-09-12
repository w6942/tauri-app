use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use enigo::{Button, Coordinate, Direction, Enigo, Mouse};

enum ReplayCommand {
    Record,
    Start { wait_time_ms: u64 },
    Stop,
}

pub struct AppState {
    command_sender: mpsc::Sender<ReplayCommand>,
}

#[tauri::command]
fn record(state: tauri::State<AppState>) -> Result<(), String> {
    println!("[Record Command] Sending RECORD signal to worker...");
    state
        .command_sender
        .send(ReplayCommand::Record)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn start_replay(
    wait_time_ms: u64,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    println!("[Start Command] Sending START signal...");
    state
        .command_sender
        .send(ReplayCommand::Start { wait_time_ms })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_replay(state: tauri::State<AppState>) -> Result<(), String> {
    println!("[Stop Command] Sending STOP signal...");
    state
        .command_sender
        .send(ReplayCommand::Stop)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = mpsc::channel::<ReplayCommand>();
    tauri::Builder::default()
        .manage(AppState { command_sender: tx }) // 注册被管理的状态
        .setup(|_| {
            thread::spawn(move || {
                let mut enigo = Enigo::new(&enigo::Settings::default()).expect("Failed to create Enigo instance");
                let mut is_replaying = false;
                let mut recorded_position: Option<(i32, i32)> = None;
                let mut wait_time_ms = 100u64;
                const MOUSE_TOLERANCE: i32 = 5;

                // 后台线程的主循环
                loop {
                    // 8. 非阻塞地检查新指令
                    if let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            ReplayCommand::Record => {
                                recorded_position = enigo.location().ok();
                                if let Some((x, y)) = recorded_position {
                                    println!("[Worker] Position recorded at ({}, {})", x, y);
                                }
                            }
                            ReplayCommand::Start { wait_time_ms: ms } => {
                                if is_replaying {
                                    println!("[Worker] Already replaying.");
                                } else if let Some((target_x, target_y)) = recorded_position {
                                    println!("[Worker] Starting replay...");

                                    // ⭐ 关键修正：在改变状态前，立即执行“首次移动”动作
                                    if let Err(e) = enigo.move_mouse(target_x, target_y, Coordinate::Abs) {
                                        eprintln!("[Worker] Failed to move mouse on start: {}", e);
                                        continue;
                                    }
                                    is_replaying = true;
                                    wait_time_ms = ms;
                                } else {
                                    println!("[Worker] No position recorded, cannot start.");
                                }
                            }
                            ReplayCommand::Stop => {
                                if is_replaying {
                                    println!("[Worker] Stopping replay.");
                                    is_replaying = false;
                                }
                            }
                        }
                    }

                    // 9. 如果处于重放状态，则执行核心逻辑
                    if is_replaying {
                        if let Some((target_x, target_y)) = recorded_position {
                            // 检查鼠标是否已移动
                            if let Ok((current_x, current_y)) = enigo.location() {
                                let dx = (current_x - target_x).abs();
                                let dy = (current_y - target_y).abs();
                                if dx > MOUSE_TOLERANCE || dy > MOUSE_TOLERANCE {
                                    println!("[Worker] Mouse moved. Stopping replay automatically.");
                                    is_replaying = false;
                                    continue;
                                }
                            }

                            // 移动并点击
                            if let Err(e) = enigo.move_mouse(target_x, target_y, Coordinate::Abs) {
                                eprintln!("[Worker] Error moving mouse: {}", e);
                                is_replaying = false; // 出错时停止
                                continue;
                            }
                            if let Err(e) = enigo.button(Button::Left, Direction::Click) {
                                eprintln!("[Worker] Error clicking mouse: {}", e);
                                is_replaying = false; // 出错时停止
                                continue;
                            }
                        }
                    }

                    // 根据状态决定休眠时间，避免CPU空转
                    let sleep_duration = if is_replaying {
                        wait_time_ms
                    } else {
                        50 // 不在重放时，可以稍微降低轮询频率
                    };
                    thread::sleep(Duration::from_millis(sleep_duration));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            record,
            start_replay,
            stop_replay
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
