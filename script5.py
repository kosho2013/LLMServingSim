import os
import multiprocessing

workload = ['GPT3_1.7B', 'GPT3_3.6B', 'GPT3_7.5B']
topology = [['perfect', 'perfect'], ['mesh', 'mesh'], ['torus', 'torus']]

def run(work, on, off):
    os.system('./run.sh System_Workload/LLM/'+work+'/ '+on+' '+off+' 16 500 > System_Workload/LLM/'+work+'/'+on+'_'+off+'_16_500.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)