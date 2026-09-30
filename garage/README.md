# Garage Usage
The S3 object storage system that this project has thus far been tested against is garage, which is a relatively light weight, open source, self hostabled S3 compatible object storage and replication system.

Garage replaces the now considered to be unmaintained minio to facilitate self hosted infrastructure.

## Setting up Garage
check configuration documentation for your specific server distribution.

To show status of current nodes:
```
garage status
garage layout assign <NODE_ID> -z $zone $capacity
garage layout apply --version 1
garage bucket create openfab
```

To create a user:
```
garage key create $keyname
garage bucket allow --key $keyid --owner --read --write
```