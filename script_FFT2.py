import os
import multiprocessing

workload = ['1M','2M','4M']
topology = [['dragonfly', 'skip']]


def run(work, on, off):
    os.system('./run.sh System_Workload/FFT/'+work+'/ '+on+' '+off+' 16 500 > System_Workload/FFT/'+work+'/'+on+'_'+off+'_16_500.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)