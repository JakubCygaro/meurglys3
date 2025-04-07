#include "meu3.h"
#include <stddef.h>
#include <stdio.h>

int main(void) {
    MEU3_Error err = -1;
    MEU3_PACKAGE* pack = meu3_package_dir("test_dir", &err);
    if(!pack) {
        return 1;
    }
    (void)meu3_package_get_compression(pack, &err);
    if (err != NoError)
        return 1;
    (void)meu3_package_get_version(pack, &err);
    if (err != NoError)
        return 1;
    size_t len = 0;
    MEU3_BYTES data = meu3_package_get_data_ptr(pack, "text.txt", &len, &err);
    if (err != NoError || len == 0)
        return 1;
    /*FILE* text = fopen("test_dir/text.txt", "r");*/
    /*if(!text)*/
    /*    return 1;*/
    /*char byte = 0;*/
    /*size_t data_idx = 0;*/
    /*while ((byte = fgetc(text)) != EOF) {*/
    /*    if(data[data_idx++] != byte) {*/
    /*        fclose(text);*/
    /*        printf("files do not match\n");*/
    /*        return 1;*/
    /*    }*/
    /*}*/
    /*fclose(text);*/
    meu3_free_package(pack);
    return 0;
}
