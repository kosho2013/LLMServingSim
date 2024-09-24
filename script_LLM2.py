import os
import multiprocessing

workload = ['7.5B_40x52']
topology = [['mesh', 'skip'], ['torus', 'skip'], ['dragonfly', 'skip']]

def run(work, on, off):
    os.system('./run.sh System_Workload/LLM/'+work+'/ '+on+' '+off+' 16 500 > System_Workload/LLM/'+work+'/'+on+'.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)


