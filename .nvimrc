function! Build() 
	:botright Topen
	:execute 'normal! i'
	:T cargo test -- --nocapture
endfunction
