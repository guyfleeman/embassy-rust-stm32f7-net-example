import socket
import struct

MCAST_GRP = '239.4.21.70'
MCAST_PORT = 42170
IS_ALL_GROUPS = False

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
if IS_ALL_GROUPS:
    # on this port, receives ALL multicast groups
    sock.bind(('', MCAST_PORT))
else:
    # on this port, listen ONLY to MCAST_GRP
    sock.bind((MCAST_GRP, MCAST_PORT))
mreq = struct.pack("4sl", socket.inet_aton(MCAST_GRP), socket.INADDR_ANY)

sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)

print("Waiting for heartbeat...")

# block waiting for board to say hello
print(sock.recv(10240))

# assume/skip heartbeat decoding

print("Hearbeat received, connect to board via TCP")
# create an INET, STREAMing socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
# now connect to the web server on port 80 - the normal http port
s.connect(("192.168.1.113", 42171))
print("Connected to board")

print(s.recv(10240))

print("Sending data response.")
s.send(b"resp")

exit(0)