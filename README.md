### What is that
Another rust playground stuff, async curl-compatible implementation

### Should I use it?
No

### But I want
Okay

### License?
Do whatever you want but don't blame me after

### Is it works as curl?
Yes, at least most options for "copy as CURL" is working

### Is it fast?
Not yet. But faster then normal curl
```
[annmuor@xhome1 currrrl]$ time curl -s https://google.com -k > /dev/null

real	0m0,279s
user	0m0,006s
sys	0m0,003s
[annmuor@xhome1 currrrl]$ time ./target/release/currrrl -s https://google.com -k > /dev/null

real	0m0,253s
user	0m0,002s
sys	0m0,002s
```
