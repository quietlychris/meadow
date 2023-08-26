import pymeadow as meadow
import time
import unittest

def main():
    
    # Create a Meadow Host
    host = meadow.Host()

    # Create Node that operates over Tcp with all the default settings
    # The topic is `gps`
    node = meadow.TcpNode("gps")

    # Publish a value; PyMeadow operates *exclusively* on String types right now
    node.publish(str(6))
    time.sleep(1)
    result = node.request()

    # If we want to do something numeric with that data we've requested, we need to convert
    # it back into the proper numeric type from the String the request() method returned      
    test = int(result.data()) + 1
    if test == 7:
        print("Success!")
    else:
        print("Failure!")

if __name__ == "__main__":
    main()
