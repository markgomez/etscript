FROM rust:1-bullseye

ARG gid=1000
ARG gname=etscript
ARG uid=1000
ARG uname=etscript

ARG dotnet_version=8.0.100-preview.6
ARG dotnet_arch=x64
ARG dotnet_root=/opt/dotnet

ENV TERM=xterm-256color \
    PATH=$dotnet_root:$PATH \
    DOTNET_ROOT=$dotnet_root \
    DOTNET_CLI_TELEMETRY_OPTOUT=1

RUN groupadd --gid $gid $gname \
    && useradd --uid $uid --gid $gid --create-home $uname \
    \
    # Init
    && apt-get update \
    && rustup component add clippy rust-src rustfmt \
    \
    # .NET
    && curl -o /opt/dotnet-sdk.tar.gz \
        $(curl -s https://dotnet.microsoft.com/en-us/download/dotnet/thank-you/sdk-$dotnet_version-linux-$dotnet_arch-binaries \
        | perl -ne "print \$1 if /.+a href=\"(.+dotnet-sdk-$dotnet_version.*-linux-$dotnet_arch.tar.gz)\"/") \
    && mkdir $dotnet_root \
    && tar -C $dotnet_root -xf /opt/dotnet-sdk.tar.gz \
    && rm /opt/dotnet-sdk.tar.gz \
    \
    # Cleanup
    && apt-get autoremove -qq \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

USER $uname:$gname

CMD ["bash"]
