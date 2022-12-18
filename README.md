# cargo-arps

```
PS D:\git\cargo-arp> cargo-arps list

        index: 40,      Hyper-V Virtual Ethernet Adapter, 172.26.32.1, 00:15:5d:76:b7:d4
        index: 15,      Microsoft Wi-Fi Direct Virtual Adapter #2, 0.0.0.0, fe:44:82:9d:3e:92
        index:  4,      Bluetooth Device (Personal Area Network), 0.0.0.0, fc:44:82:9d:3e:96
        index: 11,      Microsoft Wi-Fi Direct Virtual Adapter, 0.0.0.0, fc:44:82:9d:3e:93
        index:  9,      Realtek PCIe GbE Family Controller, 0.0.0.0, 8c:8c:aa:17:c8:43
        index:  8,      Intel(R) Wi-Fi 6 AX201 160MHz, 192.168.88.35, fc:44:82:9d:3e:92
```

```
PS D:\git\cargo-arp> cargo-arps scan 8 d

start to send arp request……
request sended, listening response……
all responses：
        9a:2a:ec:5b:8a:f4  192.168.88.163
        1c:91:80:d8:25:ce  192.168.88.175
        ……
        f0:85:c1:e2:89:58  192.168.88.71
filter result：
        e4:0c:fd:9d:c9:d5  192.168.88.188
        ……
        1c:91:80:d8:25:ce  192.168.88.175
```