<!-- 源文档：api-contract/README.md；include 保持内容单一来源。 -->
<!--@include: ../../api-contract/README.md-->

## M25–M29 增量契约

`identity` 提供安全政策、临时授权与当前后台会话再次验证；`reporting` 提供固定水位统计、差异处置与异步导出；`privacy` 提供政策、本人副本、注销核查和修复预检；`tasks` 提供独立来源启停命令。关键命令沿用原请求键及统一错误信封，下载仍需当前会话授权。可选签名事件与数组游标分页的协议见 [M25–M29 运行手册](../runbooks/m25-m29.md)。
