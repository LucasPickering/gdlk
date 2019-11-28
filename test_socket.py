#!/usr/bin/env python3

import argparse
import asyncio
import json
import websockets


async def ws(program, uri):
    with open(program) as f:
        source = f.read().strip()

    async with websockets.connect(uri) as websocket:

        async def send_and_recv(event_type, content=None):
            msg = json.dumps({"type": event_type, "content": content})
            print(f"<<< {msg}")
            await websocket.send(msg)
            recv = json.loads(await websocket.recv())
            print(f">>> {recv}")
            return recv

        await send_and_recv(event_type="edit", content={"source": source})
        state_resp = await send_and_recv(event_type="compile")

        while (
            state_resp["type"] == "machine_state"
            and not state_resp["content"]["is_complete"]
        ):
            state_resp = await send_and_recv(event_type="step")


def main():
    parser = argparse.ArgumentParser(
        description="Executes the given program over the websocket, until "
        "either error or completion"
    )
    parser.add_argument(
        "--uri",
        "-u",
        default="ws://localhost:8080/ws/",
        help="The websocket host address",
    )
    parser.add_argument(
        "--program",
        "-p",
        required=True,
        help="The file to read the program from",
    )
    args = parser.parse_args()

    asyncio.get_event_loop().run_until_complete(ws(**vars(args)))


if __name__ == "__main__":
    main()
