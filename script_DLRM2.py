import os
import multiprocessing

workload = ['0.1T','0.2T','0.4T']
topology = [['dragonfly', 'skip']]

def run(work, on, off):
    os.system('./run.sh System_Workload/DLRM/'+work+'/ '+on+' '+off+' 16 500 > System_Workload/DLRM/'+work+'/'+on+'_'+off+'_16_500.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)