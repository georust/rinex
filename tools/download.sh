#! /bin/bash
mkdir dl

final/Sta${GPS_WEEK}${DOW}.sp3
SERVER_HOSTNAME="ftp.glonass-iac.ru"
LFTP_MIRROR_OPTS="--no-empty-dirs --no-perms --no-umask --ignore-time --parallel=10"

lftp \
    -d -u anonymous, \
    -e "set ftp:ssl-force true" \
    -e "mirror ${LFTP_MIRROR_OPTS} $(echo ${SOURCE_DIRS[@]/#/--directory=}) $(echo ${INCLUDE_GLOBS[@]/#/--include-glob=}) --target-directory=dl;exit" \
    ${SERVER_HOSTNAME}
