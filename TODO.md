- D-Bus signals for
    1. Sending errors
    2. Sending updates to the state

We will be using signals for the above 2, because the daemon will be mostly
asynchronous. Once client sends an event, the client's job is done. The daemon
will do whatever with the event, and emit any state changes or errors. Then the
client will do whatever it wants with that info. Neither the daemon or the 
client is dependent on each other to complete their task.
