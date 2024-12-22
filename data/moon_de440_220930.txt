KPL/FK

   SPICE Lunar Reference Frame Specification Kernel
   =====================================================================

   Original file name:                   moon_de440_220930.tf
   Creation date:                        2022 September 30 15:05
   Created by:                           Nat Bachman  (NAIF/JPL)

   Version description:

      This frame kernel contains lunar frame specifications compatible
      with the current lunar binary PCK file

         moon_pa_de440_200625.bpc

      The above PCK contains lunar orientation data from the DE440 JPL
      Planetary Ephemeris.

      This frame kernel extends the previous version

         moon_pa_de440_200625.tf

      by adding specifications of the alias frames

         MOON_PA
         MOON_ME

      and by upgrading the documentation to include discussion of
      frame association kernels and to present an example program
      that uses this kernel.

      The description of the frame

         MOON_ME_DE440_ME421

      has been updated as well.


   Frames Specified by this Kernel
   =====================================================================

   Frame Name                  Relative to          Type   Frame ID
   -------------------------   -----------------    -----  --------
   MOON_PA                     MOON_PA_DE440        FIXED  31010
   MOON_ME                     MOON_ME_DE440_ME421  FIXED  31011
   MOON_PA_DE440               ICRF/J2000           PCK    31008
   MOON_ME_DE440_ME421         MOON_PA_DE440        FIXED  31009


   Introduction
   =====================================================================

   This kernel specifies lunar body-fixed reference frames for use by
   SPICE-based application software. These reference frames are
   associated with high-accuracy lunar orientation data provided by the
   JPL Solar System Dynamics Group's planetary ephemerides (both
   trajectory and lunar orientation data are stored in these ephemeris
   files). These ephemerides have names of the form DEnnn (DE stands
   for "developmental ephemeris").

   The frames specified by this kernel are realizations of two different
   lunar reference systems:

      Principal Axes (PA) system
      --------------------------
      The axes of this system are defined by the principal axes of the
      Moon. Due to the nature of the Moon's orbit and rotation, the Z
      axis of this system does not coincide with the Moon's mean spin
      axis, nor does the X axis coincide with the mean direction to the
      center of the Earth (in contrast with the ME system defined
      below).

      Lunar principal axes frames realizing the lunar PA system and
      specified by this kernel are associated with JPL planetary
      ephemerides. Each new JPL planetary ephemeris can (but does not
      necessarily) define a new realization of the lunar principal axes
      system. Coordinates of lunar surface features expressed in lunar
      PA frames can change slightly from one lunar ephemeris version to
      the next.


      Mean Earth/Polar Axis (ME) system
      ---------------------------------
      The Lunar mean Earth/polar axis system is a lunar body-fixed
      reference system used in the IAU/IAG Working Group Report [2] to
      describe the orientation of the Moon relative to the ICRF frame.
      The +Z axis of this system is aligned with the north mean lunar
      rotation axis, while the prime meridian contains the mean Earth
      direction.

      This system is also sometimes called the "mean Earth/mean
      rotation axis" system or "mean Earth" system.

      The mean directions used to define the axes of a mean Earth/polar
      axis reference frame realizing the lunar ME system and specified
      by this kernel are associated with a given JPL planetary
      ephemeris version. The rotation between the mean Earth frame for
      a given ephemeris version and the associated principal axes frame
      is given by a constant matrix (see [1]).

   For the current JPL planetary ephemeris DE440, this kernel includes
   specifications of the corresponding principal axes frame and a version of
   a mean Earth/polar axis frame that is closely aligned with the
   mean Earth/polar axis frame of DE421. The names of these frames are

      MOON_PA_DE440

   and

      MOON_ME_DE440_ME421

   The rotation from the former to the latter frame is described in [1]
   as the rotation "from the DE440 PA frame to DE421 MER" (mean Earth/mean
   rotation)" reference frame.

   For each of the two reference systems, there is a corresponding
   "generic" frame specification:  these generic frames are simply
   aliases for the PA and ME frames associated with the latest DE. The
   generic frame names are

      MOON_PA
      MOON_ME

   These generic frame names are provided to enable SPICE-based
   applications to refer to the latest DE-based (or other) lunar
   rotation data without requiring code modifications as new kernels
   become available. SPICE users may, if they wish, modify this kernel
   to assign these frame aliases to other frames than those selected
   here, for example, older DE-based frames. NAIF recommends that, if
   this frame kernel is modified, the name of this file also be changed
   to avoid confusion.


   Comparison of DE440 and DE421 Lunar Reference Frames
   ====================================================

   Differences in the orientation of the frames

       MOON_ME_DE421
       MOON_ME_DE440_ME421

   were measured as follows:

      For the time range

            1900 JAN 01 00:00:00.000 TDB (-3155716800.0000 TDB seconds)
         to 2051 JAN 01 00:00:00.000 TDB ( 1609416000.0000 TDB seconds)

      the maximum rotation angle, taken over ~1M samples, was

         5.8204420784597E-07 radians

      This corresponds to a displacement of slightly over 1 m on a great
      circle.

      For the time range

             2000 JAN 01 00:00:00.000 TDB (-43200.000000000 TDB seconds)
         to  2040 JAN 01 00:00:00.000 TDB ( 1262260800.0000 TDB seconds)

      the maximum rotation angle, taken over ~1M samples, was

         3.0709840342771E-07 radians

      This corresponds to a displacement of approximately 53.4 cm on a
      great circle on the Moon's surface.


   Comparison of DE440 PA and ME frames
   ====================================

   The rotation between the mean Earth frame for a given DE and the
   associated principal axes frame for the same DE is given by a
   constant matrix (see [1]). For DE440, the rotation angle between
   the MOON_ME and MOON_PA frames is approximately 0.02886 degrees;
   this is equivalent to approximately 875 m when expressed as a
   displacement along a great circle on the Moon's surface.


   Comparison of DE440 and 2009 IAU/IAG report-based ME frame
   ==========================================================

   Within the SPICE system, a lunar ME frame specified by the
   rotational elements from a IAU/IAG Working Group report such as [2]
   is given the name IAU_MOON; the data defining this frame are provided
   in a generic text PCK. Note that these data are provided for
   backward compatibility; the Working Group does not publish these
   data in later versions of the report.

   The orientation of the lunar ME frame obtained by applying the
   DE-based PA-to-ME rotation described above to the DE-based lunar
   libration data does not agree closely with the lunar ME frame
   orientation given by the rotational elements from the IAU/IAG
   Working Group report (that is, the IAU_MOON frame). The difference
   is due to truncation of the libration series used in the report's
   formula for lunar orientation (see [1]).

   In the case of DE440, for the time period 2000-2040, the
   time-dependent difference of these ME frame implementations has an
   amplitude of approximately 0.0051 degrees, which is equivalent to
   approximately 155 m, measured along a great circle on the Moon's
   surface, while the average value is approximately 0.00239 degrees,
   or 72 m.


   Regarding Use of the ICRF in SPICE
   ==================================

   The IERS Celestial Reference Frame (ICRF) is offset from the J2000
   reference frame (equivalent to EME 2000) by a small rotation:  the
   J2000 pole offset magnitude is about 18 milliarcseconds (mas) and
   the equinox offset magnitude is approximately 78 milliarcseconds
   (see [3]).

   Certain SPICE data products use the frame label "J2000" for data
   that actually are referenced to the ICRF. This is the case for SPK
   files containing JPL version DE4nn planetary ephemerides, for
   orientation data from generic text PCKs, and for binary PCKs,
   including binary lunar PCKs used in conjunction with this lunar
   frame kernel.

   Consequently, when SPICE computes the rotation between the "J2000"
   frame and either of the lunar PA or ME frames, what's computed is
   actually the rotation between the ICRF and the respective lunar
   frame.

   Similarly, when SPICE is used to compute the state given by a JPL DE
   planetary ephemeris SPK file of one ephemeris object relative to
   another (for example, the state of the Moon with respect to the
   Earth), expressed relative to the frame "J2000," the state is
   actually expressed relative to the ICRF.

   Because SPICE is already using the ICRF, users normally need not
   use the J2000-to-ICRF transformation to adjust results computed
   with SPICE.


   Lunar body-fixed frame associations
   =====================================================================

   By default, the SPICE system considers the body-fixed reference
   frame associated with the Moon to be the one named IAU_MOON. This
   body-frame association affects the outputs of the SPICE frame system
   routines

      CIDFRM
      CNMFRM

   and of the SPICE time conversion and geometry routines

      ET2LST
      ILLUM   (deprecated; superseded by ILUMIN, ILLUMF, ILLUMG)
      SRFXPT  (deprecated; superseded by SINCPT)
      SUBPT   (deprecated; superseded by SUBPNT)
      SUBSOL  (deprecated; superseded by SUBSLR)

   Also, any code that calls these routines to obtain results involving
   lunar body-fixed frames is affected. Within SPICE, the only
   higher-level subsystem that is affected is the dynamic frame
   subsystem; it currently uses SUBPT.

   NAIF provides "frame association" kernels that simplify changing the
   body-fixed frame associated with the Moon. Using FURNSH to load
   either of the kernels named below changes the Moon's body-fixed
   frame from its current value, which initially is IAU_MOON, to that
   shown in the right-hand column:

      Kernel name          Lunar body-fixed frame
      -----------          ----------------------
      moon_assoc_me.tf     MOON_ME
      moon_assoc_pa.tf     MOON_PA

   For further information see the in-line comments in the association
   kernels themselves. Also see the Frames Required Reading section
   titled "Connecting an Object to its Body-fixed Frame."

   In the N0062 SPICE Toolkit, the routines

      ILLUM
      SRFXPT
      SUBPT
      SUBSOL

   were superseded, respectively, by the routines

      ILLUMF, ILLUMG, ILUMIN
      SINCPT
      SUBPNT
      SUBSLR

   The newer routines don't require frame association kernels: the name
   of the target body's body-fixed reference frame is an input argument
   to these routines.


   Using this Kernel
   =====================================================================

   In order for a SPICE-based application to use reference frames
   specified by this kernel, the application must load both this kernel
   and a binary lunar PCK containing lunar orientation data for the
   time of interest. Normally the kernels need be loaded only once
   during program initialization.

   SPICE users may find it convenient to use a meta-kernel (also called
   a "FURNSH kernel") to list the kernels to be loaded. Below, we show
   an example of such a meta-kernel, as well as the source code of a
   small Fortran program that uses lunar body fixed frames. The
   program's output is included as well.

   The kernel names shown here are simply used as examples; users must
   select the kernels appropriate for their applications.

   Numeric results shown below may differ very slightly from those
   obtained on users' computer systems.


   Meta-kernel
   -----------

      KPL/MK

      File name: ex1.tm

      Example meta-kernel showing use of

        - binary lunar PCK
        - generic lunar frame kernel (FK)
        - leapseconds kernel (LSK)
        - planetary SPK

       30-SEP-2022 (NJB)

       Note: to actually use this kernel, replace the @ characters
       below with backslashes (\). The backslash character cannot be
       used here, within the comments of this frame kernel, because the
       begindata and begintext strings would be interpreted as
       directives bracketing actual load commands.

       This meta-kernel assumes that the referenced kernels exist
       in the user's current working directory.

          @begindata

            KERNELS_TO_LOAD = ( 'moon_pa_de440_200625.bpc'
                                'moon_de440_220930.tf'
                                'leapseconds.ker'
                                'de440.bsp'                  )

          @begintext


   Example program
   ---------------

            PROGRAM EX1
            IMPLICIT NONE

            INTEGER               FILSIZ
            PARAMETER           ( FILSIZ = 255 )

            CHARACTER*(FILSIZ)    META

            DOUBLE PRECISION      ET
            DOUBLE PRECISION      LT
            DOUBLE PRECISION      STME  ( 6 )
            DOUBLE PRECISION      STPA  ( 6 )

      C
      C     Prompt user for meta-kernel name.
      C
            CALL PROMPT ( 'Enter name of meta-kernel > ', META )

      C
      C     Load lunar PCK, generic lunar frame kernel,
      C     leapseconds kernel, and planetary ephemeris
      C     via metakernel.
      C
            CALL FURNSH ( META )

      C
      C     Convert a time of interest from UTC to ET.
      C
            CALL STR2ET ( '2022 SEP 30 TDB', ET )

            WRITE (*,*) 'ET (sec past J2000 TDB): ', ET
            WRITE (*,*) '   State of Earth relative to Moon'

      C
      C     Find the geometric state of the Earth relative to the
      C     Moon at ET, expressed relative to the ME frame.
      C
            CALL SPKEZR ( 'Earth',  ET,      'MOON_ME',
           .              'NONE',   'Moon',  STME,      LT )

            WRITE (*,*) '      In MOON_ME frame:'
            WRITE (*, '(3F12.3, 3F13.8)') STME

      C
      C     Find the geometric state of the Earth relative to the
      C     Moon at ET, expressed relative to the PA frame.
      C
            CALL SPKEZR ( 'Earth',  ET,      'MOON_PA',
           .              'NONE',   'Moon',  STPA,      LT )

            WRITE (*,*) '      In MOON_PA frame:'
            WRITE (*, '(3F12.3, 3F13.8)') STPA

            END


   Program output
   --------------

   Enter name of meta-kernel > ex1.tm
    ET (sec past J2000 TDB):    717768000.00000000
       State of Earth relative to Moon
          In MOON_ME frame:
     374008.654  -23435.970   10141.334  -0.02638134   0.04532757   0.11894164
          In MOON_PA frame:
     373997.028  -23558.987   10284.057  -0.02641181   0.04533642   0.11893151


   References
   =====================================================================

   [1]  Ryan S. Park, et al, "The JPL Planetary and Lunar Ephemerides
        DE440 and DE441," The Astronomical Journal, Volume 161, Number 3.
        2021 AJ 161 105.

        URL:

           https://iopscience.iop.org/article/10.3847/1538-3881/abd414/meta

   [2]  Archinal, B.A., A'Hearn, M.F., Bowell, E., Conrad, A.,
        Consolmagno, G.J., Courtin, R., Fukushima, T.,
        Hestroffer, D., Hilton, J.L., Krasinsky, G.A.,
        Neumann, G., Oberst, J., Seidelmann, P.K., Stooke, P.,
        Tholen, D.J., Thomas, P.C., and Williams, I.P.
        "Report of the IAU Working Group on Cartographic Coordinates
        and Rotational Elements: 2009."

   [3]  Roncoli, R. (2005). "Lunar Constants and Models Document,"
        JPL D-32296.


   Frame Specifications
   =====================================================================

   MOON_PA is the name of the generic lunar principal axes (PA) reference
   frame. This frame is an alias for the principal axes frame defined
   by the latest version of the JPL Solar System Dynamics Group's
   planetary ephemeris.

   In this instance of the lunar reference frames kernel, MOON_PA is an
   alias for the lunar principal axes frame associated with the
   planetary ephemeris DE440.

   \begindata

      FRAME_MOON_PA                 = 31010
      FRAME_31010_NAME              = 'MOON_PA'
      FRAME_31010_CLASS             = 4
      FRAME_31010_CLASS_ID          = 31010
      FRAME_31010_CENTER            = 301

      TKFRAME_31010_SPEC            = 'MATRIX'
      TKFRAME_31010_RELATIVE        = 'MOON_PA_DE440'
      TKFRAME_31010_MATRIX          = ( 1 0 0
                                        0 1 0
                                        0 0 1 )

   \begintext

   MOON_ME is the name of the generic lunar mean Earth/polar axis (ME)
   reference frame.

   In this instance of the lunar reference frames kernel, MOON_ME is an
   alias for the frame MOON_ME_DE440_ME421, which is closely aligned
   with the lunar mean Earth/polar axis frame associated with the
   planetary ephemeris DE421.

   \begindata

      FRAME_MOON_ME                 = 31011
      FRAME_31011_NAME              = 'MOON_ME'
      FRAME_31011_CLASS             = 4
      FRAME_31011_CLASS_ID          = 31011
      FRAME_31011_CENTER            = 301

      TKFRAME_31011_SPEC            = 'MATRIX'
      TKFRAME_31011_RELATIVE        = 'MOON_ME_DE440_ME421'
      TKFRAME_31011_MATRIX          = ( 1 0 0
                                        0 1 0
                                        0 0 1 )


   \begintext

      MOON_PA_DE440 is the name of the lunar principal axes reference
      frame defined by JPL's DE440 planetary ephemeris.

   \begindata

      FRAME_MOON_PA_DE440           = 31008
      FRAME_31008_NAME              = 'MOON_PA_DE440'
      FRAME_31008_CLASS             = 2
      FRAME_31008_CLASS_ID          = 31008
      FRAME_31008_CENTER            = 301

   \begintext

      MOON_ME_DE440_ME421 is the name of a reference frame defined by a
      constant rotational offset from the DE440 principal axes frame,
      such that the frame is closely aligned with the DE421 lunar mean
      Earth/polar axis frame.

      Rotation angles are from [1]. The angles and axis sequence shown
      below represent the inverse of the cited rotation; hence the
      angles are negated and the sequence is reversed.

   \begindata

      FRAME_MOON_ME_DE440_ME421        = 31009
      FRAME_31009_NAME                 = 'MOON_ME_DE440_ME421'
      FRAME_31009_CLASS                = 4
      FRAME_31009_CLASS_ID             = 31009
      FRAME_31009_CENTER               = 301

      TKFRAME_31009_SPEC            = 'ANGLES'
      TKFRAME_31009_RELATIVE        = 'MOON_PA_DE440'
      TKFRAME_31009_ANGLES          = (   67.8526   78.6944   0.2785  )
      TKFRAME_31009_AXES            = (   3,        2,        1       )
      TKFRAME_31009_UNITS           = 'ARCSECONDS'

   \begintext

   =====================================================================
   End of kernel
