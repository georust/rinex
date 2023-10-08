#! /usr/bin/env python3

import os
import tomli
import tomli_w

def parse_toml(path):
    with open(path, "r") as fd:
        content = fd.read()
        tomli_dict = tomli.loads(content)
        return tomli_dict


def parse_pool():
    tomls = []
    for subdir in os.listdir("."):
        if os.path.isdir(subdir):
            target = subdir  + "/Cargo.toml"
            if os.path.exists(target):
                tomls.append(target)
    
    content = {}
    for toml in tomls:
        key = toml.split("/")[0] # dir name
        content[key] = parse_toml(toml)
    return content

def update_pool(content):
    for subdir in os.listdir("."):
        if os.path.isdir(subdir):
            target = subdir + "/Cargo.toml"
            if os.path.exists(target):
                with open(target, "w") as fd:
                    fd.write(tomli_w.dumps(content[subdir]))

def replace_local_referencing(content):
    latest = {}
    for pkg in content:
        # latest uploaded version
        latest[pkg] = content[pkg]["package"]["version"]
    print(latest)
    for pkg in content:
        # replace local paths
        dependencies = list(content[pkg]["dependencies"].keys())
        for localpkg in latest:
            if localpkg in dependencies:
                content[pkg]["dependencies"][localpkg].pop("path", None)
                content[pkg]["dependencies"][localpkg]["version"] = latest[localpkg]
                # print(content[pkg]["dependencies"][localpkg])


def main(release=None):
    content = parse_pool()
    replace_local_referencing(content)
    update_pool(content)

if __name__ == "__main__":
    main()
