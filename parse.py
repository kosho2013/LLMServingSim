import sys

f = open(str(sys.argv[1]), 'r')
lines = f.readlines()
f.close()


cnt1 = []
cnt2 = []
for line in lines:
    if "digraph" in line:
        cnt1.append(0)
        cnt2.append(0)

    if "shape" in line:
        cnt1[-1] += 1
    if "Channel" in line:
        cnt2[-1] += 1

print("Number of contexts", cnt1)
print("Number of channels", cnt2)
