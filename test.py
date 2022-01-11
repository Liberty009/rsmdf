import ctypes
from sys import platform
import numpy as np
from cffi import FFI

ffi = FFI()
ffi.cdef("""
	typedef struct {
		double* timevalues;
		uint64_t time_length;
		double* data_values;
		uint64_t data_length;
	} TimeSeries;

	TimeSeries read_series(const char *filepath, const char *channel);

""")

lib_path = "./target/release/rsmdf.dll"

C = ffi.dlopen(lib_path)

# try: 
# 	add_lib = ctypes.CDLL(lib_path)
# 	print("Successfully loaded ", add_lib)
# except Exception as e:
# 	print(e)

file_path = "Larger_Test.mdf"
channel = "ASAM.M.SCALAR.SBYTE.IDENTICAL.DISCRETE"

filepath = file_path.encode('utf-8')
chan = channel.encode('utf-8')

file = ffi.new("char*", filepath)
channel_name = ffi.new("char*", chan)

data = C.read_series(file, 
	channel_name)

