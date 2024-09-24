import os
import multiprocessing

workload = ['2x2', '4x4', '8x8']
topology = [['perfect', 'perfect'], ['mesh', 'mesh'], ['torus', 'torus']]

def run(work, on, off):
    os.system('./run.sh System_Workload/LLM/'+work+'/ '+on+' '+off+' 16 500 > System_Workload/LLM/'+work+'/'+on+'_'+off+'_16_500.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)