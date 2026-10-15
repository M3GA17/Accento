"""Build Accento's editable native FigCraft library with the installed MCP CLI.

No server configuration, publishing, or unrelated project mutation is required.
Run: python scripts/create-figcraft-system.py
"""
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'design' / 'figcraft'
CLI = Path(os.environ.get('FIGCRAFT_CLI', r'C:\Program Files\FigCraft\figcraft-cli.exe'))


class MCP:
    def __init__(self):
        self.proc = subprocess.Popen([str(CLI), 'mcp'], stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                     text=True, encoding='utf-8')
        self.seq = 0
        self.request('initialize', {'protocolVersion': '2025-06-18', 'capabilities': {},
                     'clientInfo': {'name': 'Accento product UI builder', 'version': '0.3.0'}})
        self.proc.stdin.write(json.dumps({'jsonrpc': '2.0', 'method': 'notifications/initialized'})+'\n')
        self.proc.stdin.flush()

    def request(self, method, params):
        self.seq += 1
        self.proc.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': self.seq,
                                         'method': method, 'params': params}, ensure_ascii=False)+'\n')
        self.proc.stdin.flush()
        line = self.proc.stdout.readline()
        if not line:
            raise RuntimeError('FigCraft MCP exited before responding')
        reply = json.loads(line)
        if 'error' in reply:
            raise RuntimeError(reply['error'])
        return reply['result']

    def call(self, tool, **args):
        result = self.request('tools/call', {'name': tool, 'arguments': args})
        contents = '\n'.join(c['text'] for c in result.get('content', []) if c['type'] == 'text')
        if result.get('isError'):
            raise RuntimeError(f'{tool} {args}: {contents}')
        try:
            return json.loads(contents)
        except json.JSONDecodeError:
            return contents

    def command(self, command, **params):
        return self.call('run_command', command=command, params=params)

    def close(self):
        self.proc.stdin.close()
        self.proc.wait(timeout=15)



