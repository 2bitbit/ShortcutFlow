use crate::dialogs::pop_do_nothing_dialog;
use crate::engine;
use crate::models::DataEnvelope;
use crate::stores::flow_store::{self, get_flow_by_id};
use crate::{app_state::AppState, stores::flow_store::get_all_flows};
use anyhow::{Context, Result};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::MessageDialogKind;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// 用于在程序运行时添加快捷键、删除快捷键、为特定流更新快捷键
pub struct ShortcutManager {
    app_handle: AppHandle,
}
/// 关联函数
impl ShortcutManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

/// 对外接口
impl ShortcutManager {
    /// 要修改已有的快捷键时调用这个。（不管旧流有没有快捷键，都会返回 Ok）
    pub fn update(&self, id_flow_to_exec: &str, shortcut: Shortcut) -> Result<()> {
        // 旧流一定要能被获取，否则 update 函数就报错。
        let old_flow = get_flow_by_id(&self.app_handle.state(), &id_flow_to_exec)
            .with_context(|| format!("未能获取id为{id_flow_to_exec} 的流"))?
            .with_context(|| format!("id为{id_flow_to_exec} 的流不存在"))?;
        if let Some(old_shortcut) = old_flow.shortcut {
            self.remove_by_shortcut(old_shortcut)
                .with_context(|| format!("未能移除该流(id: {id_flow_to_exec})旧有的快捷键(旧有快捷键为:{old_shortcut:?})，尝试刷新可能解决此问题"))?;
        }
        self.register(id_flow_to_exec, shortcut).with_context(|| {
            format!("未能为流(id: {id_flow_to_exec})成功 register 新的快捷键({shortcut:?})")
        })?;
        Ok(())
    }

    /// 新增快捷键时调用这个
    pub fn add(&self, id_flow_to_exec: &str, shortcut: Shortcut) -> Result<()> {
        let all_flows =
            get_all_flows(&self.app_handle.state()).with_context(|| format!("未能获取所有的流"))?;

        for flow in all_flows.into_iter().filter(|f| f.id != id_flow_to_exec) {
            if let Some(existing_shortcut) = flow.shortcut {
                if existing_shortcut == shortcut {
                    anyhow::bail!("快捷键: {}已被其余流: {}占用", shortcut, flow.display_name)
                }
            }
        }
        self.register(id_flow_to_exec, shortcut).with_context(|| {
            format!("未能为流(id: {id_flow_to_exec})成功 register 新的快捷键({shortcut:?})")
        })?;

        Ok(())
    }

    pub fn remove_by_shortcut(&self, shortcut: Shortcut) -> Result<()> {
        self.app_handle
            .global_shortcut()
            .unregister(shortcut)
            .with_context(|| format!("尝试 unregister 快捷键 ({shortcut:?}) 时失败"))?;
        Ok(())
    }

    pub fn remove_all(&self) -> Result<()> {
        self.app_handle
            .global_shortcut()
            .unregister_all()
            .with_context(|| "unregister 所有快捷键时失败")?;
        Ok(())
    }
}

/// 私有方法
impl ShortcutManager {
    fn register(&self, id_flow_to_exec: &str, shortcut: Shortcut) -> Result<()> {
        let id_flow_to_exec = id_flow_to_exec.to_string();
        self.app_handle
            .global_shortcut()
            .on_shortcut(shortcut, move |app_ref, _sc, event| {
                // 仅在按下时触发
                if event.state == ShortcutState::Pressed {
                    let app = app_ref.clone();

                    match flow_store::get_flow_by_id(&app_ref.state(), &id_flow_to_exec) {
                        Ok(Some(flow)) => {
                            tauri::async_runtime::spawn(async move {
                                let state = app.state::<AppState>();
                                // 不可以直接anyhow::bail!，
                                // 原因：闭包是把错误传递给调用者，此处调用者是异步运行时，会静默丢弃掉。
                                // 所以：要尽可能模拟在手动点击时的执行。
                                engine::cmds::run_flow_by_id(
                                    app.clone(),
                                    state,
                                    flow.id,
                                    DataEnvelope {
                                        ..Default::default()
                                    },
                                )
                                .await
                            });
                        }
                        Ok(None) => {
                            log::error!("❌Shortcut flow execution failed: 流不存在");
                            pop_do_nothing_dialog(
                                app,
                                MessageDialogKind::Error,
                                "获取流时失败",
                                &format!("没找到流，快捷键已过期，请进行刷新"),
                            );
                        }
                        Err(e) => {
                            log::error!("❌Shortcut flow execution failed: {:?}", e);
                            pop_do_nothing_dialog(
                                app,
                                MessageDialogKind::Error,
                                "获取流时失败",
                                &format!("尝试通过id获取流失败，失败原因：{:?}", e),
                            );
                        }
                    }
                }
            })
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("HotKey already registered") {
                    anyhow::anyhow!("快捷键 {:?} 已被系统或其他软件占用", shortcut,)
                } else {
                    anyhow::anyhow!(e)
                }
            })
            .with_context(|| format!("尝试 register 快捷键 ({shortcut:?}) 时失败"))?;
        Ok(())
    }
}
