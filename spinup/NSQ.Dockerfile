# adapted from https://raw.githubusercontent.com/rongfengliang/nsq-cluster-docker-compose/master/docker-compose.yaml

version: "3.6"
services:
   nsq-admin:
     image: nsqio/nsq:v1.2.1
     command: /nsqadmin  -lookupd-http-address nsq-lookup-1:4161  -lookupd-http-address  nsq-lookup-2:4261
     ports:
     - "127.0.0.1:4171:4171"
   nsq-worker-1:
     image: nsqio/nsq:v1.2.1
     hostname: nsq-worker-1
     command: /nsqd -tcp-address 0.0.0.0:4150 -data-path /usr/local/nsq/bin/data   --http-address 0.0.0.0:4151 -lookupd-tcp-address nsq-lookup-1:4160  -lookupd-tcp-address nsq-lookup-2:4260 -broadcast-address nsq-worker-1
     volumes:
     - "/tmp/eventful/NSQ/data1:/usr/local/nsq/bin/data"
     ports:
     - "127.0.0.1:4150:4150"
     - "127.0.0.1:4151:4151"
   nsq-worker-2:
     image: nsqio/nsq:v1.2.1
     hostname: nsq-worker-2
     command: /nsqd -tcp-address 0.0.0.0:4250 -data-path /usr/local/nsq/bin/data  -http-address 0.0.0.0:4251 -lookupd-tcp-address nsq-lookup-1:4160  -lookupd-tcp-address  nsq-lookup-2:4260 -broadcast-address=nsq-worker-2
     volumes:
     - "/tmp/eventful/NSQ/data2:/usr/local/nsq/bin/data"
     ports:
     - "127.0.0.1:4250:4250"
     - "127.0.0.1:4251:4251"
   nsq-worker-3:
     image: nsqio/nsq:v1.2.1
     hostname: nsq-worker-3
     command: /nsqd -tcp-address 0.0.0.0:4350 -data-path /usr/local/nsq/bin/data  --http-address 0.0.0.0:4351 -lookupd-tcp-address nsq-lookup-1:4160  -lookupd-tcp-address  nsq-lookup-2:4260 -broadcast-address=nsq-worker-3
     volumes:
     - "/tmp/eventful/NSQ/data3:/usr/local/nsq/bin/data"
     ports:
     - "127.0.0.1:4354:4350"
     - "127.0.0.1:4355:4351"
   nsq-lookup-1:
     image: nsqio/nsq:v1.2.1
     command: /nsqlookupd   -http-address 0.0.0.0:4161  -tcp-address 0.0.0.0:4160 -broadcast-address nsq-lookup-1
     ports:
     - "127.0.0.1:4160:4160"
     - "127.0.0.1:4161:4161"
   nsq-lookup-2:
     image: nsqio/nsq:v1.2.1
     command: /nsqlookupd -http-address 0.0.0.0:4261  -tcp-address 0.0.0.0:4260 -broadcast-address nsq-lookup-2
     ports:
     - "127.0.0.1:4260:4260"
     - "127.0.0.1:4261:4261"
