
import random


pts = []

max_r = 7

for r in range(1,max_r):
    rg = [i for i in range(-r,r)]
    rg2 = [i for i in range(-r,r)]
    random.shuffle(rg)
    random.shuffle(rg2)
    for x in rg:
        for y in rg2:
            if x*x + y*y < r*r and (x,y) not in pts:
                pts += [(x,y)]

print(len(pts))
for pt in pts:
    print(f"vec2({pt[0]}.0,{pt[1]}.0),")