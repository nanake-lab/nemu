import json, socket
from pathlib import Path
NETWORK_LOG = Path(__file__).with_name('blocked-network.jsonl')
def deny(operation, target):
    with NETWORK_LOG.open('a', encoding='utf-8') as stream:
        stream.write(json.dumps({'operation':operation,'target':repr(target)},ensure_ascii=False)+'\n')
    raise RuntimeError(f'CPU smoke network forbidden: {operation} {target!r}')
def blocked_connect(self, address):
    return deny('socket.connect', address)
def blocked_connect_ex(self, address):
    return deny('socket.connect_ex', address)
def blocked_create(address, *args, **kwargs):
    return deny('socket.create_connection', address)
def blocked_resolve(host, port, *args, **kwargs):
    return deny('socket.getaddrinfo', (host,port))
def blocked_sendto(self, data, *args):
    return deny('socket.sendto', args[-1])
socket.socket.connect = blocked_connect
socket.socket.connect_ex = blocked_connect_ex
socket.socket.sendto = blocked_sendto
socket.create_connection = blocked_create
socket.getaddrinfo = blocked_resolve
