# Solana CU-Efficient Huffman Encoding Challenge

The goal of this challenge is to create a CU-efficient implementation of Huffman Encoding in a Solana program. Specifically, huffman encoding URL strings to allow for large URL strings to be passed into Solana instructions while not eating up precious bytes in the transaction.

The instruction must be able to then properly decode the message. You can validate this by adding a log that displays the decoded string and comment it out for your CU submission run.

This challenge starts when this repository is made public. The challenge ends at 5:00pm PST.

Judging criteria:
1. Low CU Usage
2. Byte-footprint of strings
3. Can it handle any URL string? Meaning any special language characters, every TLD, and all Emojis / valid URL-compatible ASCII characters must work.

Some URL examples:

```
http://localhost:3000
http://subdomain.localhost:3000
https://localhost.net
https://google.com
https://a.a
https://a.com
https://git@github.com:username/repo.git
https://a-really-long-url-that-probably-would-be-so-hard-to-actually-use-but-whatever.com
https://🦝👀🍹🌏.net
https://something.yourcooldomain.com?query_param=123&val=true
```

## How to submit

Submit a pull request against this repository with your submission code.

Your code must compile, it must contain valid tests, and your pull request message must contain a table with each URL tested and its CU and compression ratio.

Submissions will be judged based on the time submitted. If there are multiple submissions that display the same performance, the winner will be chose based on the earliest submission.
