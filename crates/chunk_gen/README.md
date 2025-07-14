A fake client that forces server to generate all chunks (macro tiles).

## How to run

Install enet headers so that `enet-sys` could compile. For example, on ubuntu, run:

```bash
sudo apt install libenet-dev
```

First, ensure the blockhead server is running at port 15151.

Then, run:

```bash
cargo run
```

If successful, you should see the following output:

```
Connected to peer: Peer { inner: 0x589dff506650, _data: PhantomData<&mut ()> }
worldWidthMacro = 512
Received chunk (0, 0)!
Received chunk (0, 1)!
Received chunk (0, 2)!
Received chunk (0, 3)!
Received chunk (0, 4)!
Received chunk (0, 5)!
...
```

It might take around 15 minutes to generate a full map.

## How it works

The server uses ENet networking library to talk with client. The server sends world id and world info to client after ENet handled hand shaking.

Here's world id:

```
--- Found UDP Packet #7: 15151 -> 62567 ---
Packet {
    header: Header {
        peer_id: 0,
        compressed: false,
        has_sent_time: true,
        session_id: 0,
        sent_time: Some(
            44738,
        ),
    },
    commands: [
        Acknowledge(
            CommandHeader {
                command: 1,
                channel_id: 255,
                reliable_sequence_number: 1,
            },
            AcknowledgeCommand {
                received_reliable_sequence_number: 2,
                received_sent_time: 44770,
            },
        ),
        SendReliable(
            CommandHeader {
                command: 6,
                channel_id: 0,
                reliable_sequence_number: 1,
            },
            SendReliableCommand {
                data_length: 246,
                data: 
                23 26 3c 3f 78 6d 6c 20 76 65 72 73 69 6f 6e 3d | #&<?xml version=
                22 31 2e 30 22 20 65 6e 63 6f 64 69 6e 67 3d 22 | "1.0" encoding="
                55 54 46 2d 38 22 3f 3e 0a 3c 21 44 4f 43 54 59 | UTF-8"?>.<!DOCTY
                50 45 20 70 6c 69 73 74 20 50 55 42 4c 49 43 20 | PE plist PUBLIC 
                22 2d 2f 2f 41 70 70 6c 65 2f 2f 44 54 44 20 50 | "-//Apple//DTD P
                4c 49 53 54 20 31 2e 30 2f 2f 45 4e 22 20 22 68 | LIST 1.0//EN" "h
                74 74 70 3a 2f 2f 77 77 77 2e 61 70 70 6c 65 2e | ttp://www.apple.
                63 6f 6d 2f 44 54 44 73 2f 50 72 6f 70 65 72 74 | com/DTDs/Propert
                79 4c 69 73 74 2d 31 2e 30 2e 64 74 64 22 3e 0a | yList-1.0.dtd">.
                3c 70 6c 69 73 74 20 76 65 72 73 69 6f 6e 3d 22 | <plist version="
                31 2e 30 22 3e 0a 3c 64 69 63 74 3e 0a 20 20 20 | 1.0">.<dict>.   
                20 3c 6b 65 79 3e 77 6f 72 6c 64 49 44 3c 2f 6b |  <key>worldID</k
                65 79 3e 0a 20 20 20 20 3c 73 74 72 69 6e 67 3e | ey>.    <string>
                33 64 37 5f 6d 6f 64 69 66 69 65 64 3c 2f 73 74 | 3d7_modified</st
                72 69 6e 67 3e 0a 3c 2f 64 69 63 74 3e 0a 3c 2f | ring>.</dict>.</
                70 6c 69 73 74 3e                               | plist>          ,
            },
        ),
        BandwidthLimit(
            CommandHeader {
                command: 10,
                channel_id: 255,
                reliable_sequence_number: 2,
            },
            BandwidthLimitCommand {
                incoming_bandwidth: 0,
                outgoing_bandwidth: 0,
            },
        ),
    ],
}
```

And world info:

```
--- Found UDP Packet #11: 15151 -> 62567 ---
Packet {
    header: Header {
        peer_id: 0,
        compressed: false,
        has_sent_time: true,
        session_id: 0,
        sent_time: Some(
            44788,
        ),
    },
    commands: [
        SendReliable(
            CommandHeader {
                command: 6,
                channel_id: 0,
                reliable_sequence_number: 2,
            },
            SendReliableCommand {
                data_length: 417,
                data: 
                01 1f 8b 08 00 00 00 00 00 00 03 75 93 5d 4f 83 | ...........u.]O.
                30 14 86 ef f7 2b 2a f7 a3 14 74 1d 06 31 2a 33 | 0....+*...t..1*3
                21 d9 07 71 a8 f1 ca 34 f4 6c 6b 04 4a 4a b3 8f | !..q...4.lk.JJ..
                7f 6f d9 34 a2 ab bd a1 9c bc cf 39 6f cf 69 a3 | .o.4.......9o.i.
                db 7d 55 a2 2d a8 56 c8 fa c6 21 ae e7 20 a8 0b | .}U.-.V...!.. ..
                c9 45 bd be 71 9e f3 c7 e1 d8 b9 8d 07 d1 45 b2 | .E..q.........E.
                78 c8 df b2 09 6a 4a d1 6a 94 3d df 4f d3 07 e4 | x....jJ.j.=.O...
                0c 31 be 6b 9a 12 30 4e f2 04 65 d3 74 99 23 93 | .1.k..0N..e.t.#.
                03 e3 c9 dc 41 ce 46 eb e6 1a e3 dd 6e e7 b2 4e | ....A.F.....n..N
                e5 16 b2 ea 84 2d ce 94 6c 40 e9 c3 d4 24 1b 1a | .....-..l@...$..
                c0 e5 9a 3b a6 cc 29 fb 2f 3b 26 ca 45 a1 e3 01 | ...;..)./;&.E...
                32 2b fa 80 43 5c 28 e0 42 47 b8 db 9f a2 0a 58 | 2+..C\(.BG.....X
                19 7b 11 3e 7e 7f 84 b0 ef 8a cc 24 87 be 78 c5 | .{.>~......$..x.
                ca 16 70 4f b6 11 eb 0d b4 3a 93 a2 d6 ee be 2f | ..pO.....:...../
                35 01 58 83 8a 89 17 8e 22 fc fd f7 0f 79 b0 91 | 5.X....."....y..
                74 34 b6 81 95 a8 a5 7a 39 1d d3 86 05 36 a8 96 | t4.....z9....6..
                4f 4c d4 b9 a8 40 9d 1d 9e f8 3e 75 29 0d ce 7a | OL...@....>u)..z
                d0 48 a5 59 39 85 2d 94 b6 42 9e ad 90 62 35 97 | .H.Y9.-..B...b5.
                d5 12 80 5b 9b 41 09 09 c8 28 08 43 1b db b2 2d | ...[.A...(.C...-
                a4 49 9f 6b b5 32 97 29 0e 38 7d af cc bd 5a 89 | .I.k.2.).8}...Z.
                2e ed 57 b0 c7 69 a6 4c 23 3b b3 99 6c ff 1b 03 | ..W..i.L#;..l...
                bd b4 1a fe 03 5b 27 71 e5 5b d1 9d 54 25 9f b3 | .....['q.[..T%..
                0a 2c 96 67 8b 24 7d 4c 27 89 c5 ee 11 eb 46 71 | .,.g.$}L'.....Fq
                36 89 cb 80 52 77 ec 79 1e f1 c3 d0 1b 07 e4 6c | 6...Rw.y.......l
                24 47 f6 55 70 bd 99 b1 42 49 ab 59 e2 f7 cc 46 | $G.Up...BI.Y...F
                f8 f4 02 22 7c 7c 1f f1 27 94 8b 15 28 b5 03 00 | ..."||..'...(...
                00                                              | .               ,
            },
        ),
    ],
}
```

Where `01` means world info data and `1f 8b` is gzip header. Decompressing this gives us:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>credit</key>
    <real>0</real>
    <key>expertMode</key>
    <false/>
    <key>highestPoint.x</key>
    <integer>1096</integer>
    <key>highestPoint.y</key>
    <integer>768</integer>
    <key>minorVersion</key>
    <integer>3</integer>
    <key>noRainTimer</key>
    <real>1227.773</real>
    <key>portalLevel</key>
    <integer>0</integer>
    <key>randomSeed</key>
    <integer>1711316399</integer>
    <key>saveID</key>
    <string>3d7_modified</string>
    <key>startPortalPos.x</key>
    <integer>10740</integer>
    <key>startPortalPos.y</key>
    <integer>520</integer>
    <key>worldName</key>
    <string>MODIFIED</string>
    <key>worldTime</key>
    <real>4377.800012990831</real>
    <key>worldWidthMacro</key>
    <integer>512</integer>
</dict>
</plist>
```

In return, the client needs to report basic info:

```
--- Found UDP Packet #9: 62567 -> 15151 ---
Packet {
    header: Header {
        peer_id: 0,
        compressed: false,
        has_sent_time: true,
        session_id: 0,
        sent_time: Some(
            44804,
        ),
    },
    commands: [
        SendReliable(
            CommandHeader {
                command: 6,
                channel_id: 0,
                reliable_sequence_number: 1,
            },
            SendReliableCommand {
                data_length: 591,
                data: 
                1f 3c 3f 78 6d 6c 20 76 65 72 73 69 6f 6e 3d 22 | .<?xml version="
                31 2e 30 22 20 65 6e 63 6f 64 69 6e 67 3d 22 55 | 1.0" encoding="U
                54 46 2d 38 22 3f 3e 0a 3c 21 44 4f 43 54 59 50 | TF-8"?>.<!DOCTYP
                45 20 70 6c 69 73 74 20 50 55 42 4c 49 43 20 22 | E plist PUBLIC "
                2d 2f 2f 41 70 70 6c 65 2f 2f 44 54 44 20 50 4c | -//Apple//DTD PL
                49 53 54 20 31 2e 30 2f 2f 45 4e 22 20 22 68 74 | IST 1.0//EN" "ht
                74 70 3a 2f 2f 77 77 77 2e 61 70 70 6c 65 2e 63 | tp://www.apple.c
                6f 6d 2f 44 54 44 73 2f 50 72 6f 70 65 72 74 79 | om/DTDs/Property
                4c 69 73 74 2d 31 2e 30 2e 64 74 64 22 3e 0a 3c | List-1.0.dtd">.<
                70 6c 69 73 74 20 76 65 72 73 69 6f 6e 3d 22 31 | plist version="1
                2e 30 22 3e 0a 3c 64 69 63 74 3e 0a 09 3c 6b 65 | .0">.<dict>..<ke
                79 3e 61 6c 69 61 73 3c 2f 6b 65 79 3e 0a 09 3c | y>alias</key>..<
                73 74 72 69 6e 67 3e 31 32 33 3c 2f 73 74 72 69 | string>123</stri
                6e 67 3e 0a 09 3c 6b 65 79 3e 69 43 6c 6f 75 64 | ng>..<key>iCloud
                49 44 3c 2f 6b 65 79 3e 0a 09 3c 73 74 72 69 6e | ID</key>..<strin
                67 3e 37 63 63 66 62 63 35 39 64 66 32 35 32 64 | g>7ccfbc59df252d
                62 37 32 64 63 33 30 64 30 66 65 61 62 32 39 34 | b72dc30d0feab294
                65 38 3c 2f 73 74 72 69 6e 67 3e 0a 09 3c 6b 65 | e8</string>..<ke
                79 3e 6c 6f 63 61 6c 3c 2f 6b 65 79 3e 0a 09 3c | y>local</key>..<
                74 72 75 65 2f 3e 0a 09 3c 6b 65 79 3e 6d 69 63 | true/>..<key>mic
                4f 72 53 70 65 61 6b 65 72 4f 6e 3c 2f 6b 65 79 | OrSpeakerOn</key
                3e 0a 09 3c 74 72 75 65 2f 3e 0a 09 3c 6b 65 79 | >..<true/>..<key
                3e 6d 69 6e 6f 72 56 65 72 73 69 6f 6e 3c 2f 6b | >minorVersion</k
                65 79 3e 0a 09 3c 69 6e 74 65 67 65 72 3e 33 3c | ey>..<integer>3<
                2f 69 6e 74 65 67 65 72 3e 0a 09 3c 6b 65 79 3e | /integer>..<key>
                70 6c 61 79 65 72 49 44 3c 2f 6b 65 79 3e 0a 09 | playerID</key>..
                3c 73 74 72 69 6e 67 3e 32 30 32 63 62 39 36 32 | <string>202cb962
                61 63 35 39 30 37 35 62 39 36 34 62 30 37 31 35 | ac59075b964b0715
                32 64 32 33 34 62 37 30 3c 2f 73 74 72 69 6e 67 | 2d234b70</string
                3e 0a 09 3c 6b 65 79 3e 75 64 69 64 4e 65 77 3c | >..<key>udidNew<
                2f 6b 65 79 3e 0a 09 3c 73 74 72 69 6e 67 3e 35 | /key>..<string>5
                37 31 33 63 62 36 38 35 37 63 32 61 32 35 64 39 | 713cb6857c2a25d9
                38 31 31 65 34 32 34 31 35 36 39 66 32 39 37 3c | 811e4241569f297<
                2f 73 74 72 69 6e 67 3e 0a 09 3c 6b 65 79 3e 76 | /string>..<key>v
                6f 69 63 65 43 6f 6e 6e 65 63 74 65 64 3c 2f 6b | oiceConnected</k
                65 79 3e 0a 09 3c 66 61 6c 73 65 2f 3e 0a 3c 2f | ey>..<false/>.</
                64 69 63 74 3e 0a 3c 2f 70 6c 69 73 74 3e 0a    | dict>.</plist>. ,
            },
        ),
    ],
}
```

To ask server to generate a chunk, you need to construct a `ClientMacroBlockRequest`:

```
--- Found UDP Packet #22: 62567 -> 15151 ---
Packet {
    header: Header {
        peer_id: 0,
        compressed: false,
        has_sent_time: true,
        session_id: 0,
        sent_time: Some(
            46462,
        ),
    },
    commands: [
        SendReliable(
            CommandHeader {
                command: 6,
                channel_id: 0,
                reliable_sequence_number: 2,
            },
            SendReliableCommand {
                data_length: 9,
                data: 
                03 4f 21 00 00 01 00 00 00                      | .O!......       ,
            },
        ),
    ],
}
```

Here:

- `03` means this is a `ClientMacroBlockRequest`.
- `4f 21 00 00` is a little-endian `u32`, which is `8527`. This is macro index. The actual macro x & y coordinate can be easily obtained by doing div & mod:

  ```py
  >>> def from_macro_index(i: int) -> tuple[int, int]:
  ...   return (i % 512, i // 512)
  ...
  >>> from_macro_index(0x214f)
  (335, 16)
  ```

  Here `512` is the `worldWidthMacro` we received in world v2.

- `01` means `createIfNotCreated`, basically telling the server to create the chunk.
- `00 00 00` are paddings.


