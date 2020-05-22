import asyncio
import itertools
import logging
import sys
from ctypes import CDLL


def load_mysql_storage():
    err = _do_call_sync("mysql_storage_init")
    if err:
        raise Exception("libmysqlstorage not initialised, aborting")


def _do_call_sync(name: str, *args):
    logger = logging.getLogger(__name__)
    logger.debug("do_call_sync: >>> name: %s, args: %s", name, args)

    err = getattr(_cdll(), name)(*args)

    logger.debug("do_call_sync: <<< %s", err)
    return err


def _cdll() -> CDLL:
    if not hasattr(_cdll, "cdll"):
        _cdll.cdll = _load_cdll()

    return _cdll.cdll


def _load_cdll() -> CDLL:
    logger = logging.getLogger(__name__)
    logger.debug("_load_cdll: >>>")

    libmysqlstorage_prefix_mapping = {"darwin": "lib", "linux": "lib", "linux2": "lib", "win32": ""}
    libmysqlstorage_suffix_mapping = {"darwin": ".dylib", "linux": ".so", "linux2": ".so", "win32": ".dll"}

    os_name = sys.platform
    logger.debug("_load_cdll: Detected OS name: %s", os_name)

    try:
        libmysqlstorage_prefix = libmysqlstorage_prefix_mapping[os_name]
        libmysqlstorage_suffix = libmysqlstorage_suffix_mapping[os_name]
    except KeyError:
        logger.error("_load_cdll: OS isn't supported: %s", os_name)
        raise OSError("OS isn't supported: %s", os_name)

    libmysqlstorage_name = "{0}mysqlstorage{1}".format(libmysqlstorage_prefix, libmysqlstorage_suffix)
    logger.debug("_load_cdll: Resolved libmysqlstorage name is: %s", libmysqlstorage_name)

    try:
        res = CDLL(libmysqlstorage_name)
        logger.debug("_load_cdll: <<< res: %s", res)
        return res
    except OSError as e:
        logger.error("_load_cdll: Can't load libindy: %s", e)
        raise e
