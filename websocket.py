#
# Simple Python Client that sends data to websocket and prints return data
#

import asyncio
import websockets


async def hello():
    uri = "ws://localhost:8080"
    async with websockets.connect(uri) as websocket:
        while 1:
            name = input("Type something ")

            await websocket.send(name)
            print(f"> {name}")

            response = await websocket.recv()
            print(f"< {response}")


if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(hello())