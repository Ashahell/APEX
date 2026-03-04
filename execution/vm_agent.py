#!/usr/bin/env python3
"""
Firecracker VM Agent

This agent runs inside a Firecracker microVM and handles execution requests
via vsock. It provides isolated, secure execution for APEX tasks.

Communication protocol:
- Listen on Unix socket at /tmp/vsock-{vm_id}.sock
- Receive JSON commands: {"command": "execute", "script": "base64_encoded", "timeout": 60}
- Return JSON results: {"success": true, "output": "...", "exit_code": 0, "stderr": ""}
"""

import asyncio
import base64
import json
import os
import signal
import socket
import sys
import time
from pathlib import Path
from typing import Optional

import loguru


VM_ID = os.environ.get("APEX_VM_ID", "default")
SOCKET_PATH = f"/tmp/vsock-{VM_ID}.sock"
LOG_FILE = f"/var/log/apex-vm-{VM_ID}.log"


loguru.logger.add(
    LOG_FILE,
    rotation="10 MB",
    retention="1 day",
    level="INFO",
    format="{time:ISO8601} | {level: {level: <8}} | {name}:{function}:{line} - {message}",
)


class VMAgent:
    """Agent that handles execution requests inside Firecracker VM."""

    def __init__(self, vm_id: str):
        self.vm_id = vm_id
        self.socket_path = SOCKET_PATH
        self.running = True
        self.start_time = time.time()

        loguru.logger.info(f"VMAgent starting for VM: {vm_id}")

    async def start(self):
        """Start the VM agent."""
        if Path(self.socket_path).exists():
            os.unlink(self.socket_path)

        server = await asyncio.start_unix_server(
            self.handle_client,
            path=self.socket_path,
        )

        os.chmod(self.socket_path, 0o666)

        boot_time = time.time() - self.start_time
        loguru.logger.info(f"VMAgent ready (boot time: {boot_time:.3f}s)")
        print(f"APEX VM Agent ready (boot: {boot_time:.3f}s)")

        async with server:
            await server.serve_forever()

    async def handle_client(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
        """Handle a client connection."""
        addr = writer.get_extra_info("peername")
        loguru.logger.debug(f"Client connected: {addr}")

        try:
            data = await reader.read(65536)
            if not data:
                return

            request_str = data.decode().strip()
            loguru.logger.debug(f"Received: {request_str[:200]}")

            try:
                request = json.loads(request_str)
            except json.JSONDecodeError as e:
                loguru.logger.error(f"Invalid JSON: {e}")
                await self._send_error(writer, f"Invalid JSON: {e}")
                return

            response = await self.process_request(request)
            await self._send_response(writer, response)

        except Exception as e:
            loguru.logger.exception(f"Error handling client: {e}")
            await self._send_error(writer, str(e))
        finally:
            writer.close()
            await writer.wait_closed()

    async def _send_response(self, writer: asyncio.StreamWriter, response: dict):
        """Send JSON response to client."""
        response_str = json.dumps(response) + "\n"
        writer.write(response_str.encode())
        await writer.drain()

    async def _send_error(self, writer: asyncio.StreamWriter, error: str):
        """Send error response."""
        await self._send_response(
            writer,
            {
                "success": False,
                "output": "",
                "exit_code": 1,
                "stderr": error,
            },
        )

    async def process_request(self, request: dict) -> dict:
        """Process an execution request."""
        command = request.get("command", "")
        script_b64 = request.get("script", "")
        timeout = request.get("timeout", 60)

        if command == "execute":
            return await self.execute_script(script_b64, timeout)
        elif command == "ping":
            return {
                "success": True,
                "output": "pong",
                "exit_code": 0,
                "stderr": "",
                "vm_id": self.vm_id,
                "uptime": time.time() - self.start_time,
            }
        elif command == "info":
            return {
                "success": True,
                "output": json.dumps(
                    {
                        "vm_id": self.vm_id,
                        "uptime": time.time() - self.start_time,
                        "socket": self.socket_path,
                    }
                ),
                "exit_code": 0,
                "stderr": "",
            }
        elif command == "shutdown":
            self.running = False
            return {
                "success": True,
                "output": "Shutting down",
                "exit_code": 0,
                "stderr": "",
            }
        else:
            return {
                "success": False,
                "output": "",
                "exit_code": 1,
                "stderr": f"Unknown command: {command}",
            }

    async def execute_script(self, script_b64: str, timeout: int) -> dict:
        """Execute a base64-encoded script."""
        try:
            script_bytes = base64.b64decode(script_b64)
            script = script_bytes.decode()
        except Exception as e:
            return {
                "success": False,
                "output": "",
                "exit_code": 1,
                "stderr": f"Failed to decode script: {e}",
            }

        loguru.logger.info(f"Executing script ({len(script)} bytes, timeout: {timeout}s)")

        try:
            proc = await asyncio.create_subprocess_shell(
                script,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )

            try:
                stdout, stderr = await asyncio.wait_for(
                    proc.communicate(),
                    timeout=timeout,
                )
            except asyncio.TimeoutError:
                proc.kill()
                await proc.wait()
                return {
                    "success": False,
                    "output": "",
                    "exit_code": -1,
                    "stderr": f"Script timed out after {timeout}s",
                }

            return {
                "success": proc.returncode == 0,
                "output": stdout.decode(errors="replace"),
                "exit_code": proc.returncode,
                "stderr": stderr.decode(errors="replace"),
            }

        except Exception as e:
            loguru.logger.exception(f"Execution failed: {e}")
            return {
                "success": False,
                "output": "",
                "exit_code": 1,
                "stderr": str(e),
            }


async def main():
    """Main entry point."""
    vm_id = sys.argv[1] if len(sys.argv) > 1 else "default"

    loguru.logger.info(f"Starting APEX VM Agent for {vm_id}")

    def signal_handler(sig, frame):
        loguru.logger.info(f"Received signal {sig}, shutting down")
        sys.exit(0)

    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)

    agent = VMAgent(vm_id)
    await agent.start()


if __name__ == "__main__":
    asyncio.run(main())
