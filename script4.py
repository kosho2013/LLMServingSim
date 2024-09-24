import os
import multiprocessing

workload = ['8K', '16K']
topology = [['perfect', 'perfect'], ['mesh', 'mesh'], ['torus', 'torus']]

def run(work, on, off):
    os.system('./run.sh System_Workload/HPL/'+work+'/ '+on+' '+off+' 16 500 > System_Workload/HPL/'+work+'/'+on+'_'+off+'_16_500.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)