const message = [_]i32{ 'h', 'e', 'l', 'l', 'o' };
export fn labeled_for() i32 {
    var count: i32 = 0;
    outer: for ([_]i32{ 1, 2, 3, 4, 5 }) |_| {
        for ([_]i32{ 1, 2, 3, 4, 5 }) |_| {
            count += 1;
            break :outer;
        }
    }
    return count;
}

export fn sum_letters(upper: i32) i32 {
    var idx: usize = 0;
    var sum: i32 = 0;
    while (idx < @intCast(usize, upper)) {
        sum += message[idx];
        idx += 1;
    }
    return sum;
}
