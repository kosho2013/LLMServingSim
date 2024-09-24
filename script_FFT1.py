import os
import multiprocessing

workload = ['16x16']
topology = [['skip', 'mesh'], ['skip', 'torus'], ['skip', 'dragonfly']]

def run(work, on, off):
    os.system('./run.sh System_Workload/FFT/'+work+'/ '+on+' '+off+' 16 64 > System_Workload/FFT/'+work+'/'+on+'_'+off+'_16_64.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)