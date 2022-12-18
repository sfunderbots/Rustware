#include "rustware/src/ersim_wrapper/ersim.h"
#include "rustware/src/main.rs.h"
#include <mtest.hpp>
//#include <snappy.h>
#include <memory>

int ersquare(int x) {
//    int foo = mysquarecpp(x);
//    auto foo = snappy::kBlockLog;
//    auto foooo = snappy::MaxCompressedLength(10);
//    auto foooo = snappy_max_compressed_length(10);
//    auto bar = kFoo;
//    auto baz = mysquarecpp2(4);
    auto bbb = mysquarecpp(3);
    return x*x;
}

