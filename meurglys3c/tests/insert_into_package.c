#include "meu3.h"

#define DATA_LEN 64

int main(void) {
    MEU3_Error err = -1;
    MEU3_PACKAGE* pack = meu3_package_dir("test_dir", &err);
    if(!pack) {
        return 1;
    }
    const char data_name[] = "inserted.bin";
    unsigned char data[DATA_LEN] = { 0 };
    for(size_t i = 0; i < DATA_LEN; i++){
        data[i] = (unsigned char)rand();
    }
    bool res = meu3_package_insert(pack, data_name, data, DATA_LEN, &err);
    if(!res)
        return 1;
    res = false;
    res = meu3_package_insert(pack, "../pro.hibited", data, DATA_LEN, &err);
    if(res)
        return 1;
    err = NoError;
    res = false;
    res = meu3_write_package("dump/inspack", pack, &err);
    if(!res || err != NoError)
        return 1;
    MEU3_PACKAGE* pack2 = meu3_load_package("dump/inspack.m3pkg", &err);
    bool has = meu3_package_has(pack2, "text.txt", &err);
    if(!has || err != NoError)
        return 1;
    has = false;
    has = meu3_package_has(pack2, data_name, &err);
    if(!has || err != NoError)
        return 1;

    unsigned long long len = 0;
    MEU3_BYTES d = meu3_package_get_data_ptr(pack2, data_name, &len, &err);
    if(!d || err != NoError)
        return 1;
    for(size_t i = 0; i < DATA_LEN; i++){
        if(data[i] != d[i])
            return 1;
    }
    meu3_free_package(pack);
    meu3_free_package(pack2);
    return 0;
}
