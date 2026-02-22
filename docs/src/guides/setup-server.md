# Setup `TheBlockheads` Server on Linux

## Download server binary

Download the server from official website: [blockheads_server171.tar.gz](https://majicdave.com/share/blockheads_server171.tar.gz).

!!! note
    As of Feb 2026, the official website is down. I have uploaded a **PATCHED** server binary onto my [gdrive](https://drive.google.com/file/d/1o3w_tw30K_Dh-yCnAqGlTkE54cTV6yNw/view?usp=drive_link). The patching process is described in the [next section](#patch-server-binary). This binary runs on Ubuntu 24.04.

    Only run the binary if you trust it's safe.

## Patch server binary

You will have to run `patchelf` on the server binary to run it on linux. You can check what libraries are missing by running `ldd` on the server binary.

Here's a reference output of an **unpatched** server binary on Ubuntu 24.04:

```bash
$ ldd blockheads_server171
        linux-vdso.so.1 (0x00007fffb8beb000)
        libgnustep-base.so.1.28 => not found
        libobjc.so.4.6 => /usr/lib/libobjc.so.4.6 (0x000073777cfc7000)
        libgnutls.so.26 => /usr/lib/libgnutls.so.26 (0x000073777cdcd000)
        libgcrypt.so.11 => /usr/lib/libgcrypt.so.11 (0x000073777cd93000)
        libxml2.so.2 => /lib/x86_64-linux-gnu/libxml2.so.2 (0x000073777cbb1000)
        libffi.so.8 => /lib/x86_64-linux-gnu/libffi.so.8 (0x000073777cba3000)
        libnsl.so.1 => /lib/x86_64-linux-gnu/libnsl.so.1 (0x000073777cb87000)
        librt.so.1 => /lib/x86_64-linux-gnu/librt.so.1 (0x000073777cb82000)
        libdl.so.2 => /lib/x86_64-linux-gnu/libdl.so.2 (0x000073777cb7d000)
        libpthread.so.0 => /lib/x86_64-linux-gnu/libpthread.so.0 (0x000073777cb78000)
        libz.so.1 => /lib/x86_64-linux-gnu/libz.so.1 (0x000073777cb5c000)
        libicui18n.so.70 => not found
        libicuuc.so.70 => not found
        libicudata.so.70 => not found
        libdispatch.so => not found
        libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6 (0x000073777ca71000)
        libstdc++.so.6 => /lib/x86_64-linux-gnu/libstdc++.so.6 (0x000073777c600000)
        libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x000073777ca41000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x000073777c200000)
        libp11-kit.so.0 => /lib/x86_64-linux-gnu/libp11-kit.so.0 (0x000073777c89d000)
        libidn2.so.0 => /lib/x86_64-linux-gnu/libidn2.so.0 (0x000073777c5de000)
        libunistring.so.5 => /lib/x86_64-linux-gnu/libunistring.so.5 (0x000073777c431000)
        libtasn1.so.6 => /lib/x86_64-linux-gnu/libtasn1.so.6 (0x000073777c885000)
        libnettle.so.8 => /lib/x86_64-linux-gnu/libnettle.so.8 (0x000073777c1ab000)
        libhogweed.so.6 => /lib/x86_64-linux-gnu/libhogweed.so.6 (0x000073777c163000)
        libgmp.so.10 => /lib/x86_64-linux-gnu/libgmp.so.10 (0x000073777c0df000)
        /lib64/ld-linux-x86-64.so.2 (0x000073777cff7000)
        libicuuc.so.74 => /lib/x86_64-linux-gnu/libicuuc.so.74 (0x000073777be00000)
        liblzma.so.5 => /lib/x86_64-linux-gnu/liblzma.so.5 (0x000073777c0ad000)
        libicudata.so.74 => /lib/x86_64-linux-gnu/libicudata.so.74 (0x000073777a000000)
```

In this case, install GNUStep, ICU, then manually patch it using `patchelf`. For libdispatch, see the next section.

### `libdispatch` to run server

In Ubuntu 24.04 `libdispatch` is not longer available in the system repositories. You must manually download and install them from these URLs:

- [`libdispatch`](https://launchpad.net/ubuntu/zesty/amd64/libdispatch0/0~svn197-3.3ubuntu2)
- [`libdispatch-dev`](https://launchpad.net/ubuntu/focal/amd64/libdispatch-dev/0~svn197-3.3ubuntu2)

## Capturing server packets

You can capture network packets to or from the server for debugging purposes using the following command:

```bash
sudo tcpdump -i any -w test.pcap port 15151
```

The packets use UDP and ENET protocol. You can use `wireshark` or write your own packet parser to analyze the packets.