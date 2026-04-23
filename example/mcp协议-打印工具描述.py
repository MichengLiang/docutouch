import asyncio
import datetime
import json
from pathlib import Path
from typing import Any

from fastmcp import Client
from fastmcp.client.transports import StdioTransport


REPO_ROOT = Path(__file__).resolve().parent.parent
# DEBUG_EXE = REPO_ROOT / "target" / "debug" / "docutouch.exe"
RELEASE_EXE = REPO_ROOT / "target" / "release" / "docutouch.exe"
# SERVER_EXE = DEBUG_EXE if DEBUG_EXE.exists() else RELEASE_EXE
SERVER_EXE = RELEASE_EXE
WORKSPACE = REPO_ROOT / "tmp" / "example_workspace"
STDERR_LOG = WORKSPACE / "diagnostics_tool_params_stderr.log"


class UI:
    HEADER = "\033[95m"
    BLUE = "\033[94m"
    CYAN = "\033[96m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    BOLD = "\033[1m"
    RESET = "\033[0m"

    @staticmethod
    def section(title: str):
        print(f"\n{UI.HEADER}{'=' * 76}")
        print(f" {title}")
        print(f"{'=' * 76}{UI.RESET}")

    @staticmethod
    def ok(msg: str):
        print(f"{UI.GREEN}[OK] {msg}{UI.RESET}")

    @staticmethod
    def warn(msg: str):
        print(f"{UI.YELLOW}[WARN] {msg}{UI.RESET}")

    @staticmethod
    def err(msg: str):
        print(f"{UI.RED}[ERR] {msg}{UI.RESET}")

    @staticmethod
    def info(key: str, value: str):
        print(f"{UI.BLUE}{key:<20}{UI.RESET}: {value}")

    @staticmethod
    def code_block(title: str, text: str):
        print(f"\n{UI.CYAN}--- {title} ---{UI.RESET}")
        print(text)
        print(f"{UI.CYAN}{'-' * 40}{UI.RESET}")


def format_schema_value(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, (dict, list)):
        return json.dumps(value, ensure_ascii=False, indent=2)
    return str(value)


def show_tool_metadata(tool: Any):
    print(f"\n{UI.CYAN}🔧 工具名称: {UI.BOLD}{tool.name}{UI.RESET}")
    description = (tool.description or "").strip() or "无描述"
    print(f"   功能描述: {description}")

    schema = getattr(tool, "inputSchema", None)

    if not schema:
        print("   🚫 参数定义: 无参数")
        return

    if hasattr(schema, "model_dump"):
        schema = schema.model_dump(exclude_none=True)
    elif hasattr(schema, "dict") and callable(schema.dict):
        schema = schema.dict(exclude_none=True)
    elif not isinstance(schema, dict) and hasattr(schema, "__dict__"):
        schema = schema.__dict__

    if not isinstance(schema, dict):
        print("   ⚠️ 参数定义无法解析，原始内容如下：")
        print(f"   {schema}")
        return

    properties = schema.get("properties", {})
    required_fields = set(schema.get("required", []))

    if not properties:
        print("   🚫 参数定义: 无参数")
        return

    print("   👉 参数列表 (Schema):")
    for prop_name, prop_info in properties.items():
        if not isinstance(prop_info, dict):
            prop_info = {"raw": prop_info}

        prop_type = prop_info.get("type", "any")
        prop_desc = prop_info.get("description", "暂无描述")
        default_val = prop_info.get("default", None)
        enum_vals = prop_info.get("enum")
        examples = prop_info.get("examples")

        req_mark = (
            f"{UI.RED}[必填]{UI.RESET}"
            if prop_name in required_fields
            else f"{UI.GREEN}[选填]{UI.RESET}"
        )

        print(f"      • {UI.YELLOW}{prop_name}{UI.RESET} ({prop_type}) {req_mark}")
        print(f"        描述: {prop_desc}")

        if default_val is not None:
            print(f"        默认值: {format_schema_value(default_val)}")

        if enum_vals:
            print(f"        可选值: {format_schema_value(enum_vals)}")

        if examples:
            print(f"        示例: {format_schema_value(examples)}")

        extra_keys = [
            "items",
            "anyOf",
            "oneOf",
            "allOf",
            "format",
            "minimum",
            "maximum",
            "minLength",
            "maxLength",
            "pattern",
        ]
        extras = {k: prop_info[k] for k in extra_keys if k in prop_info}
        if extras:
            print(f"        其他约束: {format_schema_value(extras)}")

    additional_properties = schema.get("additionalProperties", None)
    if additional_properties is not None:
        print(f"   额外字段允许: {format_schema_value(additional_properties)}")


async def main():
    UI.section("DocuTouch Tool Parameter Inspector")
    UI.info("Repo Root", str(REPO_ROOT))
    UI.info("Server EXE", str(SERVER_EXE))
    UI.info("Workspace", str(WORKSPACE))
    UI.info("Stderr Log", str(STDERR_LOG))

    if not SERVER_EXE.exists():
        UI.err("没有找到 docutouch.exe，请先编译 target/release 或 target/debug")
        return

    WORKSPACE.mkdir(parents=True, exist_ok=True)

    transport = StdioTransport(
        command=str(SERVER_EXE),
        args=[],
        keep_alive=False,
        log_file=STDERR_LOG,
    )

    client = Client(
        transport=transport,
        name="DocuTouch Tool Parameter Inspector",
        timeout=datetime.timedelta(seconds=60),
    )

    try:
        async with client:
            UI.ok("已连接到 MCP 服务端")

            UI.section("列出工具并打印参数定义")
            tools = await client.list_tools()
            UI.info("Tool Count", str(len(tools)))

            for tool in tools:
                show_tool_metadata(tool)

            UI.section("完成")

    except Exception as exc:
        UI.err(f"运行失败: {exc}")
        UI.warn("可以查看 stderr 日志进一步排查")


if __name__ == "__main__":
    asyncio.run(main())