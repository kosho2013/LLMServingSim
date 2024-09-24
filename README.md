# DAMSim

## How to run
`./run.sh a b c d e`

a: input file which contains DFModel mapping<br />
b: on-chip topology<br />
c: off-chip topology<br />
d: on-chip number of streamed tiles/samples<br />
e: off-chip number of streamed tiles/samples<br />


## Mapping format:<br />
PCU:<br />
**| pcu | on_chip_config | on_chip_config ID | kernel name | location X | location Y | SIMD or Systolic | sender ID in the connection list | receiver ID in the connection |**<br />
pcu on_chip_config 0 Step_1 0 1 128 Systolic sender 0 1 2 receiver 6 

PMU:<br />
**| pmu | on_chip_config | on_chip_config ID | tensor name | location X | location Y | num of vectors written/read | sender ID in the connection list | receiver ID in the connection |**<br />
pmu on_chip_config 0 Step_1___Step_2 0 0 128 sender 8 receiver 0 

Traffic:<br />
**| connection | on_chip_config | on_chip_config ID | pcu or pmu | location X | location Y | pcu or pmu | location X | location Y |**<br />
connection on_chip_config 0 pcu 0 1 pmu 0 0





