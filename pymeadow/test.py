import pymeadow as meadow
import time
import unittest

def main():
    
    # Create a Meadow Host
    host = meadow.Host()

    # Create Node that operates over Tcp with all the default settings
    # The topic is `gps`
    node = meadow.TcpNode("gps")

    # Publish a value, which can be any Python type
    node.publish(6)
    time.sleep(1)
    # Request methods always return a String, so we'll need to do that conversion explicitly
    result = node.request()
   
    test = int(result.data()) + 1
    if test == 7:
        print("Success!")
    else:
        print("Failure!")

if __name__ == "__main__":
    main()
