"""Test local stdio réel, sans client externe ni configuration globale."""
import json
import queue
import subprocess
import threading
import time
import os
from pathlib import Path

BINARY = Path(__file__).resolve().parents[1] / 'target/debug' / ('systempulse-mcp.exe' if os.name == 'nt' else 'systempulse-mcp')

class Client:
    def __init__(self):
        self.process = subprocess.Popen([str(BINARY)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.queue = queue.Queue()
        def reader():
            for line in self.process.stdout:
                try:
                    self.queue.put(json.loads(line))
                except (ValueError, UnicodeDecodeError):
                    self.queue.put({'invalid_stdout': True})
        self.thread = threading.Thread(target=reader, daemon=True)
        self.thread.start()
    def send(self, message):
        self.process.stdin.write(json.dumps(message).encode() + b'\n')
        self.process.stdin.flush()
    def receive(self):
        message = self.queue.get(timeout=8)
        assert 'invalid_stdout' not in message, 'stdout must be JSON protocol only'
        return message
    def call(self, name, args, ident):
        self.send({'jsonrpc': '2.0', 'id': ident, 'method': 'tools/call', 'params': {'name': name, 'arguments': args}})
        result = self.receive()
        assert result.get('id') == ident, result
        return result
    def close(self):
        self.process.stdin.close()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.terminate()
            self.process.wait(timeout=5)
            raise AssertionError('server did not stop on EOF')
        assert self.process.returncode == 0, 'server shutdown failed'

started = time.monotonic()
for restart in range(2):
    client = Client()
    try:
        client.send({'jsonrpc':'2.0', 'id':1, 'method':'initialize', 'params':{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'systempulse-local-test','version':'1'}}})
        initialized = client.receive()
        assert 'result' in initialized, initialized
        protocol = initialized['result']['protocolVersion']
        client.send({'jsonrpc':'2.0','method':'notifications/initialized'})
        client.send({'jsonrpc':'2.0','id':2,'method':'tools/list','params':{}})
        tools = client.receive()['result']['tools']
        assert len(tools) == 10
        assert all(t['annotations']['readOnlyHint'] and t['inputSchema']['additionalProperties'] is False and 'outputSchema' in t for t in tools)
        if restart:
            assert 'result' in client.call('get_collector_status',{},3)
            continue
        time.sleep(3.2)
        for ident, tool in enumerate(tools, 10):
            result = client.call(tool['name'],{},ident)
            if tool['name'] == 'get_top_processes':
                assert result['result']['isError'] is True
            else:
                assert not result['result'].get('isError',False), (tool['name'],result)
                assert 'structuredContent' in result['result']
                assert len(json.dumps(result)) < 300_000
        for ident, (name, args) in enumerate([
            ('get_top_processes',{'limit':51}),
            ('get_memory_status',{'path':'/etc/passwd'}),
            ('get_cpu_telemetry',{'command':'touch /tmp/unauthorized'}),
            ('get_health_history',{'limit':0}),
            ('get_health_history',{'start_ms':100,'end_ms':1}),
        ],30):
            assert client.call(name,args,ident)['result']['isError'] is True
        for ident in range(50,58):
            client.send({'jsonrpc':'2.0','id':ident,'method':'tools/call','params':{'name':'get_collector_status','arguments':{}}})
        assert {client.receive()['id'] for _ in range(8)} == set(range(50,58))
        # Oversized input must not poison the next bounded request.
        client.process.stdin.write(b' ' * (70 * 1024) + b'\n')
        client.process.stdin.flush()
        assert 'result' in client.call('get_collector_status',{},60)
        print(f'PASS legacy handshake={protocol}; discovery=10; calls/privacy/parameters/concurrency/oversize')
    finally:
        client.close()
print(f'PASS restart + EOF shutdown; elapsed={time.monotonic()-started:.3f}s')
