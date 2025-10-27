import trio
import pyrstest

print(pyrstest.__dir__())


async def main():
    while True:
        print("waiting")
        await pyrstest.sleep(1)


if __name__ == "__main__":
    trio.run(main)
    trio.open_memory_channel
