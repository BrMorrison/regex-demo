import re
import sys
import time

regex = re.compile(r"[\w]+://[^/\s?#]+(:?\?[^\s#]*)?(:?#[^\s]*)?|[\w\.+-]+@[\w\.-]+\.[\w\.-]+|(:?(:?25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(:?25[0-5]|2[0-4]\d|[01]?\d\d?)")

input_file = sys.argv[1]
text = ""
with open(input_file, 'r') as f:
    text = f.read()

    matches = []
    start = time.time()
    for line in text.split('\n'):
        if regex.search(line):
            matches.append(line)
    elapsed = time.time() - start

    print(f"{len(matches)} matches in {elapsed} s")
    for line in matches:
        print(line)