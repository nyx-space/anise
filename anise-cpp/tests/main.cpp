#include <iostream>
#include <cassert>
#include <string>
#include "anise-cpp/src/lib.rs.h"

int main() {
    auto epoch = anise::time::epoch_from_str("2023-01-01T00:00:00 UTC");
    std::cout << "Epoch: " << std::string(epoch->to_string()) << std::endl;

    auto duration = anise::time::duration_from_seconds(3600.0);
    std::cout << "Duration (s): " << duration->total_seconds() << std::endl;

    assert(duration->total_seconds() == 3600.0);

    auto start = anise::time::epoch_from_tai_seconds(0.0);
    auto end = anise::time::epoch_from_tai_seconds(3600.0);
    auto step = anise::time::duration_from_seconds(600.0);

    auto series = anise::time::time_series_new(*start, *end, *step);
    int count = 0;
    while (series->has_next()) {
        auto e = series->next();
        std::cout << "Series Epoch: " << std::string(e->to_string()) << std::endl;
        count++;
    }
    std::cout << "Count: " << count << std::endl;
    assert(count == 7); // 0 to 3600 inclusive every 600s is 0, 600, 1200, 1800, 2400, 3000, 3600

    return 0;
}
