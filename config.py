import truststore

truststore.inject_into_ssl()

TDNET_BASE_URL = "https://www.release.tdnet.info/inbs/"
TDNET_LIST_URL = TDNET_BASE_URL + "I_list_{page:03d}_{date}.html"

DEFAULT_PORT = 8080
