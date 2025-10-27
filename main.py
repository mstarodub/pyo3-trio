import trio
import pyrstest

print(pyrstest.__dir__())
print(pyrstest.sum_as_string(1, 2))


async def main():
    while True:
        print("waiting")
        # await trio.sleep(1)
        await pyrstest.sleep(1)


if __name__ == "__main__":
    trio.run(main)
    trio.open_memory_channel
