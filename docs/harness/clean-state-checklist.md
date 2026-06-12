# Clean State Checklist

结束重要会话前检查：

- [ ] 标准启动路径仍然可用：`bash init.sh`
- [ ] `docs/harness/feature_list.json` 是有效 JSON
- [ ] 当前进度已经写入 `docs/harness/progress.md`
- [ ] 功能状态真实反映 passing、blocked、in_progress 或 not_started
- [ ] 未完成工作已经写入 `docs/harness/session-handoff.md`
- [ ] 没有未记录的半成品步骤
- [ ] 没有提交密钥、私有 cookie、个人 token 或大型二进制资源
- [ ] 下一轮会话无需聊天上下文即可继续
