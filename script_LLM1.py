import os
import multiprocessing

workload = ['16x16']
topology = [['skip', 'torus'], ['skip', 'mesh'], ['skip', 'dragonfly']]

def run(work, on, off):
    os.system('./run.sh System_Workload/LLM/'+work+'/ '+on+' '+off+' 16 128 > System_Workload/LLM/'+work+'/'+on+'_'+off+'_16_128.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)
