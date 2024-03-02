complete -c rwpspread -s i -l image -d 'Image file path' -r
complete -c rwpspread -s a -l align -d 'Do not downscale the base image, align the layout instead' -r -f -a "{tl	'',tr	'',tc	'',bl	'',br	'',bc	'',rc	'',lc	'',c	''}"
complete -c rwpspread -s b -l backend -d 'Wallpaper setter backend' -r -f -a "{wpaperd	'',swaybg	'',hyprpaper	''}"
complete -c rwpspread -s d -l daemon -d 'Enable daemon mode, will watch and resplit on output changes'
complete -c rwpspread -s p -l palette -d 'Generate a color palette from input image'
complete -c rwpspread -s s -l swaylock -d 'Use swaylock integration'
complete -c rwpspread -s f -l force-resplit -d 'Force resplit, skips all image cache checks'
complete -c rwpspread -s h -l help -d 'Print help'
complete -c rwpspread -s V -l version -d 'Print version'
