import socket
import time

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)

while True:
    print("sending packet...")
    sock.sendto(b"robot", ("192.168.1.113", 42170))

    # print(sock.recv(1024))

    time.sleep(1)