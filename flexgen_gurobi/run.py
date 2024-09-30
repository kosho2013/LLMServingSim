import gurobipy as gp
import argparse
import numpy as np
import pprint
from enum import Enum
import pydot
import copy
import os





model = gp.Model()
model.params.NonConvex = 2
model.params.MIPGap = 0.001
model.Params.Threads = os.cpu_count()



# system
gmem = 16*1024**3 # B
cmem = 208*1024**3 # B
dmem = 1.5*1024*1024**3 # B
ctog_bdw = 12*1024**3 # B/s
gtoc_bdw = 12*1024**3 # B/s
dtoc_bdw = 2*1024**3 # B/s
ctod_bdw = 2*1024**3 # B/s
mm_flops = 65*1e12 # FLOPS
bmm_flops = 65*1e12 # FLOPS
cpu_flops = 1.024*1e12 # FLOPS

# LLM model
s = 512
n = 32


# OPT 175
l = 96
h1 = 12288
h2 = 49152

# OPT 30B
# l = 48
# h1 = 7168
# h2 = 28672

# OPT 6.7B
# l = 32
# h1 = 4096
# h2 = 16384


wg = model.addVar(name='wg', vtype=gp.GRB.CONTINUOUS)
wc = model.addVar(name='wc', vtype=gp.GRB.CONTINUOUS)
wd = model.addVar(name='wd', vtype=gp.GRB.CONTINUOUS)
cg = model.addVar(name='cg', vtype=gp.GRB.CONTINUOUS)
cc = model.addVar(name='cc', vtype=gp.GRB.CONTINUOUS)
cd = model.addVar(name='cd', vtype=gp.GRB.CONTINUOUS)
hg = model.addVar(name='hg', vtype=gp.GRB.CONTINUOUS)
hc = model.addVar(name='hc', vtype=gp.GRB.CONTINUOUS)
hd = model.addVar(name='hd', vtype=gp.GRB.CONTINUOUS)
bls = model.addVar(name='bls', vtype=gp.GRB.INTEGER, lb=1)
gbs = model.addVar(name='gbs', vtype=gp.GRB.INTEGER, lb=1)
num_blocks = model.addVar(name='num_blocks', vtype=gp.GRB.INTEGER, lb=1)

T = model.addVar(name='T', vtype=gp.GRB.CONTINUOUS)
Tpre = model.addVar(name='Tpre', vtype=gp.GRB.CONTINUOUS)
Tgen = model.addVar(name='Tgen', vtype=gp.GRB.CONTINUOUS)
latency_per_request = model.addVar(name='latency_per_request', vtype=gp.GRB.CONTINUOUS)

ctogp = model.addVar(name='ctogp', vtype=gp.GRB.CONTINUOUS)
gtocp = model.addVar(name='gtocp', vtype=gp.GRB.CONTINUOUS)
dtocp = model.addVar(name='dtocp', vtype=gp.GRB.CONTINUOUS)
ctodp = model.addVar(name='ctodp', vtype=gp.GRB.CONTINUOUS)
compp = model.addVar(name='compp', vtype=gp.GRB.CONTINUOUS)

ctogg = model.addVar(name='ctogg', vtype=gp.GRB.CONTINUOUS)
gtocg = model.addVar(name='gtocg', vtype=gp.GRB.CONTINUOUS)
dtocg = model.addVar(name='dtocg', vtype=gp.GRB.CONTINUOUS)
ctodg = model.addVar(name='ctodg', vtype=gp.GRB.CONTINUOUS)
compg = model.addVar(name='compg', vtype=gp.GRB.CONTINUOUS)
gpu_compg = model.addVar(name='gpu_compg', vtype=gp.GRB.CONTINUOUS)
cpu_compg = model.addVar(name='cpu_compg', vtype=gp.GRB.CONTINUOUS)



gpu_home = model.addVar(name='gpu_home', vtype=gp.GRB.CONTINUOUS)
gpu_w = model.addVar(name='gpu_w', vtype=gp.GRB.CONTINUOUS)
cpu_home = model.addVar(name='cpu_home', vtype=gp.GRB.CONTINUOUS)
cpu_w = model.addVar(name='cpu_w', vtype=gp.GRB.CONTINUOUS)
disk_home = model.addVar(name='disk_home', vtype=gp.GRB.CONTINUOUS)






model.addConstr(bls == 256)
# model.addConstr(gbs == 32)
# model.addConstr(num_blocks == 8)

# model.addConstr(wc == 0.5)
# model.addConstr(wd == 0.5)
# model.addConstr(cd == 1)
# model.addConstr(hc == 1)




# model.addConstr(wg >= 0.01)
# model.addConstr(wc >= 0.01)
# model.addConstr(wd >= 0.01)
# model.addConstr(cg >= 0.01)
# model.addConstr(cc >= 0.01)
# model.addConstr(cd >= 0.01)
# model.addConstr(hg >= 0.01)
# model.addConstr(hc >= 0.01)
# model.addConstr(hd >= 0.01)









model.addConstr(bls == gbs * num_blocks)
model.addConstr(T == Tpre * l + Tgen * (n - 1) * l)


model.addConstr(Tpre >= ctogp)
model.addConstr(Tpre >= gtocp)
model.addConstr(Tpre >= dtocp)
model.addConstr(Tpre >= ctogp)
model.addConstr(Tpre >= compp)

model.addConstr(ctogp * ctog_bdw == ((wc + wd) * (8*h1**2 + 4*h1 * h2) + 2 * (hc + hd) * h1 * bls))
model.addConstr(gtocp * gtoc_bdw == (4 * (cc + cd) * (s) * h1 * bls + 2 * (hc + hd) * h1 * bls))
model.addConstr(dtocp * dtoc_bdw == (wd * (8*h1**2 + 4*h1 * h2) + 2 * hd * s * h1 * bls))
model.addConstr(ctodp * ctod_bdw == (4 * cd * bls * (s) * h1 + 2 * hd * s * h1 * bls))
model.addConstr(compp == (bls * (8*s * h1**2 + 4*s * h1 * h2) / mm_flops) + (4*bls * s**2 * h1 / bmm_flops))










model.addConstr(Tgen >= ctogg)
model.addConstr(Tgen >= gtocg)
model.addConstr(Tgen >= dtocg)
model.addConstr(Tgen >= ctogg)
model.addConstr(Tgen >= compg)

model.addConstr(ctogg * ctog_bdw == ((wc + wd) * (8*h1**2 + 4*h1 * h2) + 2 * (hc + hd) * h1 * bls))
model.addConstr(gtocg * gtoc_bdw == 2 * (hc + hd) * h1 * bls)
model.addConstr(dtocg * dtoc_bdw == (4*cd*bls*(s)*h1 + wd*(8*h1**2 + 4*h1*h2) + 2*hd*h1*bls))
model.addConstr(ctodg * ctod_bdw == (4*cd*bls*h1 + 2*hd*h1*bls))
model.addConstr(compg == gpu_compg + cpu_compg)
model.addConstr(gpu_compg == (bls * (8*h1**2 + 4*h1*h2) / mm_flops) + (4*cg * bls * (s) * h1 / bmm_flops))
model.addConstr(cpu_compg == 4 * (cc + cd) * bls * (s) * h1 / cpu_flops)







model.addConstr(gpu_home == wg * (8*h1**2 + 4*h1*h2) * l + hg * 2*s * h1 * bls + 4 * (s) * h1 * cg * bls * l)
model.addConstr(gpu_w == 2 * (1 - wg) * (8*h1**2 + 4*h1*h2) + (1 - hg) * 2 * s * h1 * bls)
model.addConstr(cpu_home == wc * (8*h1**2 + 4*h1*h2) * l + hc * 2 * s * h1 * bls + 4 * (s) * h1 * cc * bls * l)
model.addConstr(cpu_w == wd * (8*h1**2 + 4*h1*h2) + (1-hg) * 2 * h1 * s * bls + (1-cg) * 4 * (s) * h1 * bls)
model.addConstr(disk_home == (8*h1**2 + 4*h1*h2) * wd * l + hd * 2 * s * h1 * bls + cd * 4 * (s) * h1 * bls * l)



model.addConstr(gpu_home + gpu_w <= gmem)
model.addConstr(cpu_home + cpu_w <= cmem)
model.addConstr(disk_home <= dmem)

model.addConstr(wg + wc + wd == 1.0)
model.addConstr(cg + cc + cd == 1.0)
model.addConstr(hg + hc + hd == 1.0)











tokens_per_s = model.addVar(name='tokens_per_s', vtype=gp.GRB.CONTINUOUS)
model.addConstr(tokens_per_s * T == bls * n)


model.addConstr(latency_per_request * bls == T)


model.setObjective(latency_per_request, gp.GRB.MINIMIZE)
model.optimize()





# get variable values from gurobi program
for v in model.getVars():
    print(v.varName, v.x)

    if v.varName.startswith("bls"):
        bls = v.x
    if v.varName.startswith("wg"):
        wg = v.x
    if v.varName.startswith("wc"):
        wc = v.x
    if v.varName.startswith("wd"):
        wd = v.x
    if v.varName.startswith("cg"):
        cg = v.x
    if v.varName.startswith("cc"):
        cc = v.x
    if v.varName.startswith("cd"):
        cd = v.x
    if v.varName.startswith("hg"):
        hg = v.x
    if v.varName.startswith("hc"):
        hc = v.x
    if v.varName.startswith("hd"):
        hd = v.x


print('------------------------------- memory transfer ----------------------------------------')
print('initialize')
print('')

for i in range(n):
    for j in range(l):
        if i == 0: # prefill
            print('prefill: token', i, 'layer', j)


        else: # decode
            print('decode: token', i, 'layer', j)
        
