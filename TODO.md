I need to make coreipc more reliable and less of a "non-rust" mess. I am using
a bunch of unwraps, and expecting the consumer of the api to keep a track of
the state. Well, I am the consumer of the API and I dont trust myself with that.

And I also need to make the messaging bits more bi-directional and robust 
against reconnects. I should also be able to support multiple clients. The plan
is to have a proper API to communicate with the daemon. Right now I do it in a
very hacky manner

Well, I will take a small break from this, I need to finish my Advent of Code
for last year, as I have not yet done it all. ESPECIALLY day 21. Maybe I will
feel less like a failure if I end up completing all 50 stars, even if they are
a little late...
