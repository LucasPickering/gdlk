#!/usr/bin/env python3

import argparse
import asyncio
import json
import websockets


async def ws(host, env_id, source):
    with open(source) as f:
        source = f.read().strip()

    async with websockets.connect(
        f"{host}/ws/environments/{env_id}/"
    ) as websocket:

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
        "--host", default="ws://localhost:8000", help="The server host address"
    )
    parser.add_argument(
        "--env-id",
        "-e",
        default=1,
        help="The ID for the environment to execute under",
    )
    parser.add_argument(
        "--source",
        "-s",
        required=True,
        help="The file to read the program source from",
    )
    args = parser.parse_args()

    asyncio.get_event_loop().run_until_complete(ws(**vars(args)))


if __name__ == "__main__":
    main()
