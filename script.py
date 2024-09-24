import os
import multiprocessing

# A = ['DLRM_0.1T'] 
A = ['1K', '2K', '4K', '8K']

def run_exp1(a): 
    os.system('./run.sh System_Workload/HPL/'+a+'/ 1_16 > System_Workload/HPL/'+a+'/1_16.txt')

def run_exp2(a): 
    os.system('./run.sh System_Workload/HPL/'+a+'/ 2_16 > System_Workload/HPL/'+a+'/2_16.txt')


programs = []
for a in A:
    run_exp1(a)

for a in A:
    run_exp2(a)

    