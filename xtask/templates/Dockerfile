FROM {{ image }}:{{ version }}
ARG DOWNLOAD_AUTH_TOKEN=""
ARG DOWNLOAD_RELEASE_ID=""
ARG DOWNLOAD_VERSION=""
ENV CT_DOWNLOAD_AUTH_TOKEN=${DOWNLOAD_AUTH_TOKEN}
ENV CT_DOWNLOAD_RELEASE_ID=${DOWNLOAD_RELEASE_ID}
ENV CT_DOWNLOAD_VERSION=${DOWNLOAD_VERSION}
{% if image == TestOs::Ubuntu || image == TestOs::Debian -%}
# fixes apt-get issue on Debian and Ubuntu images where packages were missing
RUN apt-get update
{% else if image == TestOs::RockyLinux -%}
# fixes package conflict in rockylinux 9+ images
# curl-minimal is already installed
# but our install script tries to install curl
RUN yum install -y --allowerasing curl
{% endif -%}
COPY ./install.sh /tmp/ctinstall.sh
RUN chmod 755 /tmp/ctinstall.sh && /tmp/ctinstall.sh \
    --version "${CT_DOWNLOAD_VERSION}" \
    ${CT_DOWNLOAD_RELEASE_ID:+ --auth-token "${CT_DOWNLOAD_AUTH_TOKEN}" --release-id "${CT_DOWNLOAD_RELEASE_ID}"}
CMD cloudtruth --version