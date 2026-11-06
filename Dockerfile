# Production Container Image for MSI Mystic Light Controller for Linux
FROM ubuntu:24.04

LABEL org.opencontainers.image.title="MSI Mystic Light Controller for Linux" \
      org.opencontainers.image.description="High-performance controller and GUI for MSI MysticLight 24-zone and 4-zone keyboard backlights" \
      org.opencontainers.image.url="https://github.com/anas-kummali/msi-mysticlight-controller-linux" \
      org.opencontainers.image.source="https://github.com/anas-kummali/msi-mysticlight-controller-linux" \
      org.opencontainers.image.version="1.2.0" \
      org.opencontainers.image.licenses="GPL-3.0-only"

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libxkbcommon0 \
        libgl1 \
        libwayland-client0 \
        libx11-6 \
        libxcursor1 \
        libxi6 \
        libxrandr2 \
        udev \
    && rm -rf /var/lib/apt/lists/*

COPY target/release/msi-mysticlight /usr/local/bin/msi-mysticlight
COPY target/release/msi-mysticlight-gui /usr/local/bin/msi-mysticlight-gui
COPY packaging/99-msi-mysticlight.rules /etc/udev/rules.d/99-msi-mysticlight.rules

ENTRYPOINT ["msi-mysticlight"]
CMD ["--status"]
