import random
import socket
from time import sleep
from datetime import datetime, timezone
import aprs
import argparse

BUFFER_SIZE = 1024
MYCALL = "NOCALL"

FEND = b"\xc0"
FESC = b"\xdb"
TFEND = b"\xdc"
TFESC = b"\xdd"
ENC_FEND = b"\xdb\xdc"
ENC_FESC = b"\xdb\xdd"


def _encode_special(data):
    if data.find(FESC) >= 0:
        data = data.replace(FESC, ENC_FESC)
    if data.find(FEND) >= 0:
        data = data.replace(FEND, ENC_FEND)
    return data

def lat_enc(lat:float):
    if lat<0:
        return str(lat)+"W"
    return str(lat)+"E"

def encode_frame(port, command, data):
    frame = bytearray()
    frame.extend(FEND)
    frame.append(command | port << 4)
    if data:
        frame.extend(_encode_special(data))
    frame.extend(FEND)
    return frame

lats = [4041.48]
lons = [-07400.95]
frames = [
    aprs.APRSFrame.ui(
        destination="APRS",
        source="KD4AAA-1",
        path=["WIDE2-2"],
        info=(
            "/"
            + datetime.now(tz=timezone.utc).strftime("%d%H%M")
            + "z"

            + r"4041.48N/07400.95W#O095/041/A=020729"
        ).encode("ascii"),
    ),
    aprs.APRSFrame.ui(
        destination="APRS",
        source="KC1QXQ-1",
        path=["WIDE2-2"],
        info=(
            "/"
            + datetime.now(tz=timezone.utc).strftime("%d%H%M")
            + r"z4221.78N/07107.54WWO095/041/A=020729"
        ).encode("ascii"),
    ),
    aprs.APRSFrame.ui(
        destination="APRS",
        source="KD4AAA-3",
        path=["WIDE2-2"],
        info=(
            "/"
            + datetime.now(tz=timezone.utc).strftime("%d%H%M")
            + r"z4221.78N/07107.54W#O095/041/A=020729"
        ).encode("ascii"),
    ),
]


n_frames = len(frames)


def kiss_escape(data: bytes) -> bytes:
    data = data.replace(b"\xdb", b"\xdb\xdd")
    data = data.replace(b"\xc0", b"\xdb\xdc")
    return data


def run_kiss_server(address: (str, int), loop: bool, randomize:bool):
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    print(f"KISS TCP server listening on {address[0]}:{address[1]}")
    server_socket.bind((address))
    server_socket.listen(10)

    while True:
        print("Waiting for a connection...")
        connection, client_address = server_socket.accept()
        print(f"Connection from {client_address}")
        i = 0
        while loop or i < len(frames):
            print("iter")
            try:
                if randomize:
                    frame = frames[random.randrange(n_frames)]
                else:
                    frame = frames[i]
                    i += 1
                connection.send(encode_frame(0, int(0x00), bytes(frame)))
            except Exception as e:
                print(f"An error occurred: {e}")
                connection.close()
                break
            finally:
                sleep(0.5)
        connection.close()
        if not loop:    
            break
        print("done")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="TestAPRS", description="APRS test packet sender for KISS over TCP"
    )
    parser.add_argument(
        "-r",
        "--random",
        dest="randomize",
        action="store_true",
        help="randomize which packet is sent from set",
    )
    parser.add_argument(
        "-l",
        "--loop",
        dest="loop",
        action="store_true",
        help="send repeatedly until ctrl-c is provided",
    )
    parser.add_argument(
        "-a",
        "--address",
        dest="address",
        type=str,
        help="host address",
        default="0.0.0.0",
    )
    parser.add_argument(
        "-p", "--port", dest="port", type=int, help="host port", default="8001"
    )
    args = parser.parse_args()

    run_kiss_server((args.address, args.port), args.loop, args.randomize)
