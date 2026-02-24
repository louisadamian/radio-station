import random
import socket
from time import sleep
from datetime import datetime, timezone
import aprs
TCP_IP = '0.0.0.0'
TCP_PORT = 8001
BUFFER_SIZE = 1024
MYCALL = "NOCALL"

FEND  = b'\xC0'
FESC  = b'\xDB'
TFEND = b'\xDC'
TFESC = b'\xDD'
ENC_FEND = b'\xDB\xDC'
ENC_FESC = b'\xDB\xDD'

def _encode_special(data):
    if data.find(FESC) >= 0:
        data = data.replace(FESC, ENC_FESC)
    if data.find(FEND) >= 0:
        data = data.replace(FEND, ENC_FEND)
    return data
def encode_frame(port, command, data):
    frame = bytearray()
    frame.extend(FEND)
    frame.append(command | port << 4)
    if data:
        frame.extend(_encode_special(data))
    frame.extend(FEND)
    return frame




info=("/" +datetime.now(tz=timezone.utc).strftime("%d%H%M")+r"z4041.48N/07490.96W&O095/041/A=020729").encode('ascii')
frames = [
aprs.APRSFrame.ui(
    destination="APRS",
    source="KD4AAA-1",
    path=["WIDE2-2"],
    info=("/" + datetime.now(tz=timezone.utc).strftime("%d%H%M") + r"z4041.48N/07400.95W#O095/041/A=020729").encode(
        'ascii')
), aprs.APRSFrame.ui(
    destination="APRS",
    source="KC1QXQ-1",
    path=["WIDE2-2"],
    info=("/" + datetime.now(tz=timezone.utc).strftime("%d%H%M") + r"z4221.78N/07107.54WWO095/041/A=020729").encode(
        'ascii')
), aprs.APRSFrame.ui(
    destination="APRS",
    source="KD4AAA-3",
    path=["WIDE2-2"],
    info=("/" + datetime.now(tz=timezone.utc).strftime("%d%H%M") + r"z4221.78N/07107.54W#O095/041/A=020729").encode(
        'ascii')
),
]


n_frames = len(frames)

def kiss_escape(data: bytes) -> bytes:
    data = data.replace(b'\xDB', b'\xDB\xDD')
    data = data.replace(b'\xC0', b'\xDB\xDC')
    return data

def run_kiss_server():
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server_socket.bind((TCP_IP, TCP_PORT))
    server_socket.listen(10)
    print(f"KISS TCP server listening on {TCP_IP}:{TCP_PORT}")

    while True:
        print("Waiting for a connection...")
        connection, client_address = server_socket.accept()
        print(f"Connection from {client_address}")
        # connection.send(encode_frame(0,0x00, bytes(frame)))
        while True:
            print("iter")
            try:
                frame = frames[random.randrange(n_frames)]
                print(frames)
                connection.send(encode_frame(0, int(0x00), bytes(frame)))
            except Exception as e:
                print(f"An error occurred: {e}")
                connection.close()
                break
            finally:
                sleep(.5)
        print("done")
if __name__ == '__main__':
    run_kiss_server()
