# HOST-01 请求本地 user_text 来源授权接口

HOST-01 需要由权限实现提供一个生产谓词，供 `CoreHandler<Rehearsal>::authorize_event` 调用。输入包括网关建立的本机会话、事件和来源元数据；需要从可信侧解析的绑定由权限实现取得。输出为允许本条文字观察或明确的 `IngressReason`。请给出已有实现位置，或由权限卡提供接口和允许、拒绝两侧的用例。

P5-05 的 `confirmation` 选择工具确认方式，`validate_binding` 核对行动批准。P5-40 的 `check_dispatch_binding` 接受执行请求，`check_management_binding` 只接受成长管理请求。W18 的 `validate_event` 在来源授权之后才执行。终端声明的认识论类型和安装令牌本身均无法完成这一步。

在接口可用前，宿主返回 `CAPABILITY_DENIED`。验收时要用同一个生产 handler，从脚本终端演示合法输入的持久回执以及未经授权来源的拒绝。

已处理。[交叉审查决定](https://github.com/nanake-lab/nemu/pull/258#issuecomment-7080901147)指定由本卡实现 Windows 账户来源绑定和文字谓词。[握手补充决定](https://github.com/nanake-lab/nemu/pull/258#issuecomment-7080949946)授权在返回认证成功前持久化。实现与测试见本 PR。
